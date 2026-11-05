"""Exercise the installed native extension, not a Python mock."""

import doctest
import errno
import gc
import inspect
import io
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

import ddes
from ddes import (
    AssayRecord,
    AssaysFile,
    AssaysMetadata,
    DdesArchive,
    IntensityFile,
    IntensityMetadata,
    IntensityRecord,
    MainFile,
    MainMetadata,
    MainRecord,
    read_assays,
    read_intensity,
    read_main,
    write_assays,
    write_intensity,
    write_main,
)

CORE = Path(__file__).resolve().parents[2] / "ddes-rs"
FIXTURES = CORE / "tests" / "fixtures"
MAIN = (FIXTURES / "DDES_expID_run_202506301127_main.tsv").read_bytes()
ASSAYS = (FIXTURES / "DDES_expID_run_202506301127_assays.tsv").read_bytes()
INTENSITY = (FIXTURES / "DDES_expID_run_202506301127_intensity_A1.tsv").read_bytes()


class BindingsTest(unittest.TestCase):
    def test_public_docstrings_and_examples(self):
        # PyO3 builtins are not reliably discovered by doctest.testmod().
        # Read the installed extension's docs, not Rust source or .pyi stubs.
        objects = [("ddes", ddes), ("ddes._native", ddes._native)]
        for name, value in vars(ddes).items():
            if (
                name.startswith("_")
                or getattr(value, "__module__", None) != "ddes._native"
            ):
                continue
            objects.append((f"ddes.{name}", value))
            if inspect.isclass(value):
                objects.extend(
                    (f"ddes.{name}.{member}", getattr(value, member))
                    for member in vars(value)
                    if not member.startswith("_")
                )
        runner = doctest.DocTestRunner()
        output = io.StringIO()
        for name, value in objects:
            with self.subTest(api=name):
                doc = value.__doc__
                self.assertTrue(
                    doc and doc.strip(), f"Missing runtime docstring: {name}"
                )
                if callable(value):
                    inspect.signature(value)
                example = doctest.DocTestParser().get_doctest(doc, {}, name, None, 0)
                runner.run(example, out=output.write)
        self.assertGreater(runner.tries, 0, "No executable docstring examples found")
        self.assertEqual(runner.failures, 0, output.getvalue())

    def test_assays_missing_lists_and_validation(self):
        file = read_assays(ASSAYS)
        self.assertIsInstance(file, AssaysFile)
        self.assertEqual(
            file.metadata,
            AssaysMetadata(
                ddes_version="1.0",
                datetime="202506301127",
                experiment_id="expID",
                run_id="5160815c-092d-49ea-946c-cf7bcb49432a",
                ddes_type="assays",
            ),
        )
        self.assertEqual(
            file.records,
            [
                AssayRecord(
                    "HIV Triplex", "PSI.GREEN;ENV.YELLOW;GAG.CRIMSON", "125;125;125"
                )
            ],
        )
        for kind in ["assay", "assays"]:
            file.metadata.ddes_type = kind
            for value in ["NA", "0;NA;2.5", "NA;NA;NA", "1e2;0.25;3"]:
                with self.subTest(kind=kind, concentrations=value):
                    file.records[0].probe_concentrations_nm = value
                    file.records[0].assay_id = '001\t"β"\n#assay'
                    self.assertEqual(read_assays(write_assays(file)), file)
        for value in ["", "-1", "125;;3", "NaN", "Inf", "1e400", "125;bad;3"]:
            with self.subTest(concentrations=value):
                file.records[0].probe_concentrations_nm = value
                with self.assertRaises(ValueError):
                    write_assays(file)
                with self.assertRaises(ValueError):
                    read_assays(ASSAYS.replace(b"125;125;125", value.encode()))
        for value in [None, 125, [125, None, 0]]:
            with self.subTest(value=value), self.assertRaises(TypeError):
                file.records[0].probe_concentrations_nm = value

    def test_main_native_numbers_missing_values_and_exact_u64(self):
        file = read_main(MAIN)
        self.assertIsInstance(file, MainFile)
        self.assertIsInstance(file.metadata, MainMetadata)
        self.assertEqual(file.metadata.instrument, "QIAcuity Four")
        self.assertEqual(
            file.records[0],
            MainRecord(
                well_id="A1",
                sample_id="Sample1",
                assay_id="HIV Triplex",
                target_id="PSI",
                counts_positive=4236,
                counts_negative=21173,
                concentration_reaction_cp_ul=241.9,
                threshold_if_threshold=18.88,
            ),
        )
        for positive, negative in [
            (2**64 - 1, 2**53 + 1),
            (None, 0),
            (0, None),
            (None, None),
        ]:
            with self.subTest(positive=positive, negative=negative):
                row = file.records[0]
                row.counts_positive = positive
                row.counts_negative = negative
                row.concentration_reaction_cp_ul = None
                row.threshold_if_threshold = -2.5
                row.sample_id = '001\t"µ"\n#sample'
                output = write_main(file)
                self.assertIsInstance(output, bytes)
                self.assertEqual(read_main(output), file)
                if positive == 2**64 - 1:
                    self.assertIn(b'"18446744073709551615"\t"9007199254740993"', output)
        file.records = []
        self.assertEqual(read_main(write_main(file)), file)

    def test_main_rejects_invalid_counts_and_concentrations(self):
        row = read_main(MAIN).records[0]
        for key in ["counts_positive", "counts_negative"]:
            for value, error in [
                (-1, OverflowError),
                (2**64, OverflowError),
                (1.5, TypeError),
                ("12", TypeError),
                ("NA", TypeError),
            ]:
                with self.subTest(key=key, value=value), self.assertRaises(error):
                    setattr(row, key, value)
        for key, values in [
            ("concentration_reaction_cp_ul", [-0.1, float("nan"), float("inf")]),
            ("threshold_if_threshold", [float("nan"), float("inf"), float("-inf")]),
        ]:
            for value in values:
                with self.subTest(key=key, value=value):
                    file = read_main(MAIN)
                    setattr(file.records[0], key, value)
                    with self.assertRaises(ValueError):
                        write_main(file)
        for key in [
            "counts_positive",
            "counts_negative",
            "concentration_reaction_cp_ul",
            "threshold_if_threshold",
        ]:
            file = read_main(MAIN)
            with self.assertRaises(AttributeError):
                delattr(file.records[0], key)

    def test_intensity_floats_and_normalized_roundtrip(self):
        file = read_intensity(INTENSITY)
        self.assertIsInstance(file, IntensityFile)
        self.assertEqual(
            file.metadata,
            IntensityMetadata(
                ddes_version="1.0",
                datetime="202506301127",
                experiment_id="expID",
                run_id="5160815c-092d-49ea-946c-cf7bcb49432a",
                ddes_type="intensity",
                well_id="A1",
                assay_id="HIV Triplex",
            ),
        )
        self.assertEqual(len(file.records), 9)
        self.assertEqual(
            file.records[0],
            IntensityRecord(partition_id="9", channel_id="CRIMSON", fi_endpoint=34.65),
        )
        for value in [
            0.0,
            -12.75,
            sys.float_info.max,
            float.fromhex("0x0.0000000000001p-1022"),
        ]:
            file.records[0].fi_endpoint = value
            file.records[0].partition_id = '0001\t"β"\n#partition'
            self.assertEqual(read_intensity(write_intensity(file)), file)
        for value in [None, "12.5"]:
            with self.subTest(value=value), self.assertRaises(TypeError):
                file.records[0].fi_endpoint = value
        for value in [float("nan"), float("inf"), float("-inf")]:
            with self.subTest(value=value):
                file.records[0].fi_endpoint = value
                with self.assertRaises(ValueError):
                    write_intensity(file)
        for token in [b"", b"NA", b"NaN", b"Inf", b"-", b"1e400"]:
            with (
                self.subTest(token=token),
                self.assertRaisesRegex(ValueError, r"intensity.tsv, line 11"),
            ):
                read_intensity(INTENSITY.replace(b"34.65", token, 1))
        file.records = []
        self.assertEqual(read_intensity(write_intensity(file)), file)

    def test_class_construction_and_nested_edits(self):
        main = MainFile(
            MainMetadata(
                ddes_version="1.0.0",
                datetime="now",
                experiment_id="experiment",
                run_id="run",
                ddes_type="main",
                instrument="test",
                plate_type="plate",
                instrument_software_version="1",
            ),
            [],
        )
        intensity = IntensityFile(read_intensity(INTENSITY).metadata, [])
        assays = AssaysFile(
            AssaysMetadata(
                ddes_version="1.0.0",
                datetime="later",
                experiment_id="assay-experiment",
                run_id="assay-run",
                ddes_type="assay",
            ),
            [],
        )
        constructed = DdesArchive(main, assays, [intensity])
        restored = DdesArchive.from_bytes(constructed.to_bytes())
        self.assertEqual(restored.main(), main)
        self.assertEqual(restored.assays(), assays)
        self.assertEqual(restored.intensities(), [intensity])
        for file, row, read, write in [
            (
                assays,
                AssayRecord("007", "PSI.GREEN;GAG.CRIMSON", "0;NA"),
                read_assays,
                write_assays,
            ),
            (
                main,
                MainRecord(
                    well_id="B2",
                    sample_id="007",
                    assay_id="assay",
                    target_id="target",
                    counts_positive=0,
                    counts_negative=None,
                    concentration_reaction_cp_ul=None,
                    threshold_if_threshold=-1.5,
                ),
                read_main,
                write_main,
            ),
            (
                intensity,
                IntensityRecord("001", "BLUE", -3.5),
                read_intensity,
                write_intensity,
            ),
        ]:
            file.records.append(row)
            self.assertIs(file.records[0], row)
            metadata = file.metadata
            metadata.experiment_id = "edited"
            self.assertEqual(read(write(file)), file)
            file.records[0] = row
            self.assertEqual(read(write(file)).records, [row])
            del file.records[0]
            self.assertEqual(read(write(file)).records, [])
            file.records = [row]
            self.assertEqual(read(write(file)).records, [row])
            file.records.append({})
            with self.assertRaises(TypeError):
                write(file)
        with self.assertRaises(TypeError):
            IntensityRecord("1", "BLUE")
        with self.assertRaises(TypeError):
            write_intensity(main)

    def test_errors_and_unknown_fields_are_not_silently_discarded(self):
        for read, write, data in [
            (read_main, write_main, MAIN),
            (read_assays, write_assays, ASSAYS),
            (read_intensity, write_intensity, INTENSITY),
        ]:
            for section in ["file", "metadata", "record"]:
                file = read(data)
                target = (
                    file
                    if section == "file"
                    else file.metadata
                    if section == "metadata"
                    else file.records[0]
                )
                with self.assertRaises(AttributeError):
                    target.unexpected = "keep me"
            with self.assertRaises(TypeError):
                write({})
            file = read(data)
            file.metadata.ddes_type = "invalid"
            with self.assertRaisesRegex(ValueError, "DDES_type"):
                write(file)
        with self.assertRaisesRegex(ValueError, "ZIP"):
            DdesArchive.from_bytes(b"not a zip")
        with tempfile.TemporaryDirectory() as directory:
            missing = Path(directory) / "missing.ddes"
            with self.assertRaises(FileNotFoundError) as error:
                DdesArchive.read_path(missing)
            self.assertEqual(error.exception.errno, errno.ENOENT)
            self.assertEqual(error.exception.filename, str(missing))

    def test_archive_updates_copy_inputs_and_replace_intensities_atomically(self):
        main = read_main(MAIN)
        assays = read_assays(ASSAYS)
        a = read_intensity(INTENSITY)
        b = read_intensity(INTENSITY)
        b.metadata.well_id = "B2"
        b.records[0].fi_endpoint = -8.25
        archive = DdesArchive(main, assays, [b, a])
        main.records[0].counts_positive = 11
        assays.records[0].probe_concentrations_nm = "0;NA;2.5"
        a.records[0].fi_endpoint = 19.5
        self.assertEqual(archive.main(), read_main(MAIN))
        self.assertEqual(archive.assays(), read_assays(ASSAYS))
        self.assertEqual(archive.intensity(1), read_intensity(INTENSITY))
        self.assertEqual(
            [f.metadata.well_id for f in archive.intensities()], ["B2", "A1"]
        )

        archive.set_main(main)
        archive.set_assays(assays)
        archive.set_intensities([b, a])
        restored = DdesArchive.from_bytes(archive.to_bytes())
        self.assertEqual(restored.main(), main)
        self.assertEqual(restored.assays(), assays)
        # Reads sort by well ID; both distinct measurements survive serialization.
        self.assertEqual(restored.intensities(), [a, b])
        main.records[0].counts_positive = 99
        assays.records[0].probe_concentrations_nm = "NA"
        a.records[0].fi_endpoint = 88.0
        self.assertEqual(archive.main().records[0].counts_positive, 11)
        self.assertEqual(
            archive.assays().records[0].probe_concentrations_nm, "0;NA;2.5"
        )
        self.assertEqual(archive.intensity(1).records[0].fi_endpoint, 19.5)
        snapshot = archive.intensities()
        snapshot[0].records.clear()
        snapshot.clear()
        self.assertEqual(archive.intensity_count, 2)
        self.assertEqual(len(archive.intensity(0).records), 9)

        # A late conversion failure must not leave the first replacement applied.
        b.records[0].fi_endpoint = 123.75
        a.records.append({})
        with self.assertRaises(TypeError):
            archive.set_intensities([b, a])
        self.assertEqual(
            archive.intensities(), [restored.intensity(1), restored.intensity(0)]
        )
        for file, setter, getter in [
            (main, archive.set_main, archive.main),
            (assays, archive.set_assays, archive.assays),
        ]:
            before = getter()
            file.records.append({})
            with self.assertRaises(TypeError):
                setter(file)
            self.assertEqual(getter(), before)

        archive.set_intensities([b])
        self.assertEqual(DdesArchive.from_bytes(archive.to_bytes()).intensities(), [b])
        archive.set_intensities([])
        self.assertEqual(archive.intensity_count, 0)
        self.assertEqual(DdesArchive.from_bytes(archive.to_bytes()).intensities(), [])
        with self.assertRaises(TypeError):
            DdesArchive(read_main(MAIN), read_assays(ASSAYS), [main])

    def test_archive_write_path_validates_before_creation_and_never_overwrites(self):
        archive = DdesArchive(read_main(MAIN), read_assays(ASSAYS), [])
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / "résultats.ddes"
            archive.write_path(destination)
            self.assertEqual(
                DdesArchive.read_path(destination).assays(), read_assays(ASSAYS)
            )
            original = destination.read_bytes()
            with self.assertRaises(FileExistsError) as error:
                archive.write_path(str(destination))
            self.assertEqual(error.exception.errno, errno.EEXIST)
            self.assertEqual(error.exception.filename, str(destination))
            self.assertEqual(destination.read_bytes(), original)
            missing = Path(directory) / "missing" / "output.ddes"
            with self.assertRaises(FileNotFoundError) as error:
                archive.write_path(missing)
            self.assertEqual(error.exception.errno, errno.ENOENT)
            self.assertEqual(error.exception.filename, str(missing))

            invalid_destination = Path(directory) / "invalid.ddes"
            bad_assays = archive.assays()
            bad_assays.records[0].probe_concentrations_nm = "-1"
            archive.set_assays(bad_assays)
            for write in [
                archive.to_bytes,
                lambda: archive.write_path(invalid_destination),
            ]:
                with self.assertRaises(ValueError):
                    write()
            self.assertFalse(invalid_destination.exists())
            archive.set_assays(read_assays(ASSAYS))
            a = read_intensity(INTENSITY)
            for files in [[a, a], [a]]:
                if len(files) == 1:
                    a.metadata.well_id = "../escape"
                archive.set_intensities(files)
                with self.assertRaises(ValueError):
                    archive.write_path(invalid_destination)
                self.assertFalse(invalid_destination.exists())

    def test_example_checks_roundtrips_with_and_without_optimization(self):
        example = Path(__file__).resolve().parents[1] / "examples" / "roundtrip.py"
        for flags in [[], ["-O"]]:
            with self.subTest(flags=flags), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / "example.ddes"
                result = subprocess.run(
                    [sys.executable, *flags, str(example), str(FIXTURES), str(output)],
                    capture_output=True,
                    text=True,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("Verified edited archive round trip", result.stdout)
                restored = DdesArchive.read_path(output)
                self.assertEqual(restored.main().records[0].counts_positive, 2**64 - 1)
                self.assertEqual(
                    restored.assays().records[0].probe_concentrations_nm, "NA"
                )
                self.assertEqual(restored.intensity(0).records[0].fi_endpoint, -12.75)
                # A changed parsed value must fail verification even under -O.
                result = subprocess.run(
                    [
                        sys.executable,
                        *flags,
                        "-c",
                        """
import runpy
import sys
from pathlib import Path
namespace = runpy.run_path(sys.argv[1])
main = namespace["main"]
read_main = main.__globals__["read_main"]
def changed_read(data):
    file = read_main(data)
    file.records[0].sample_id = "unexpected sample"
    return file
main.__globals__["read_main"] = changed_read
main(Path(sys.argv[2]))
""",
                        str(example),
                        str(FIXTURES),
                    ],
                    capture_output=True,
                    text=True,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertNotIn("Verified edited archive round trip", result.stdout)

    def test_archive_path_bytes_ownership_and_native_roundtrip(self):
        with tempfile.TemporaryDirectory() as directory:
            native = Path(directory) / "résultats.ddes"
            subprocess.run(
                [
                    "cargo",
                    "run",
                    "--locked",
                    "--quiet",
                    "--example",
                    "roundtrip",
                    "--",
                    str(FIXTURES),
                    str(native),
                ],
                cwd=CORE,
                check=True,
                capture_output=True,
            )
            archive = DdesArchive.read_path(native)
            self.assertEqual(archive.intensity_count, 1)
            self.assertEqual(archive.main(), read_main(MAIN))
            self.assertEqual(archive.assays(), read_assays(ASSAYS))
            self.assertEqual(archive.intensity(0), read_intensity(INTENSITY))
            self.assertEqual(DdesArchive.read_path(FIXTURES).main(), archive.main())
            main = archive.main()
            assays = archive.assays()
            intensity = archive.intensity(0)
            main.records[0].counts_positive = 2**64 - 1
            assays.records[0].probe_concentrations_nm = "NA"
            intensity.records[0].fi_endpoint = 777.0
            self.assertEqual(archive.main().records[0].counts_positive, 4236)
            self.assertEqual(archive.intensity(0).records[0].fi_endpoint, 34.65)
            self.assertEqual(
                archive.assays().records[0].probe_concentrations_nm, "125;125;125"
            )
            for index in [-1, 1]:
                with self.assertRaises(IndexError):
                    archive.intensity(index)
            with self.assertRaises(TypeError):
                archive.intensity(0.5)
            with self.assertRaises(OverflowError):
                archive.intensity(2**100)
            output = archive.to_bytes()
            del archive
            gc.collect()
            self.assertEqual(read_main(write_main(main)), main)
            self.assertEqual(read_assays(write_assays(assays)), assays)
            self.assertEqual(read_intensity(write_intensity(intensity)), intensity)
            self.assertEqual(DdesArchive.from_bytes(output).main(), read_main(MAIN))
            # Compare every normalized component.
            with (
                zipfile.ZipFile(native) as left,
                zipfile.ZipFile(io.BytesIO(output)) as right,
            ):
                self.assertEqual(
                    {name: left.read(name) for name in left.namelist()},
                    {name: right.read(name) for name in right.namelist()},
                )
            python_output = Path(directory) / "python.ddes"
            edited = DdesArchive(main, assays, [intensity])
            edited.write_path(python_output)
            result = subprocess.run(
                [
                    "cargo",
                    "run",
                    "--locked",
                    "--quiet",
                    "--example",
                    "roundtrip",
                    "--",
                    str(python_output),
                    str(Path(directory) / "native-reloaded.ddes"),
                ],
                cwd=CORE,
                check=True,
                capture_output=True,
                text=True,
            )
            self.assertIn("Verified semantic round trip", result.stdout)
            native_restored = DdesArchive.read_path(
                Path(directory) / "native-reloaded.ddes"
            )
            self.assertEqual(native_restored.main(), main)
            self.assertEqual(native_restored.assays(), assays)
            self.assertEqual(native_restored.intensities(), [intensity])


if __name__ == "__main__":
    unittest.main()
