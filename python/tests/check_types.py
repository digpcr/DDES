"""Static API checks; unused ignores fail if the installed stubs weaken."""

from pathlib import Path

from ddes import (
    AssayRecord,
    AssaysFile,
    AssaysMetadata,
    DdesArchive,
    IntensityFile,
    IntensityRecord,
    MainFile,
    read_assays,
    read_intensity,
    read_main,
    write_assays,
    write_intensity,
    write_main,
)


class CustomRecord(IntensityRecord):  # type: ignore[misc]
    pass


def check_api(data: bytes, path: Path) -> bytes:
    main: MainFile = read_main(data)
    main.records[0].counts_positive = 2**64 - 1
    main.records[0].counts_negative = None
    main.records[0].counts_positive = 1.5  # type: ignore[assignment]
    main.records[0].counts_negative = "NA"  # type: ignore[assignment]
    main.metadata.experiment_ID = "wrong naming"  # type: ignore[attr-defined]
    write_main(main)
    intensity: IntensityFile = read_intensity(data)
    intensity.records[0].fi_endpoint = -12.75
    intensity.records[0].fi_endpoint = None  # type: ignore[assignment]
    intensity.records.append(IntensityRecord("10", "BLUE", 4.5))
    intensity.records.append(main.records[0])  # type: ignore[arg-type]
    write_intensity(main)  # type: ignore[arg-type]
    write_intensity({})  # type: ignore[arg-type]
    write_intensity(intensity)
    assays: AssaysFile = read_assays(data)
    assays.metadata = AssaysMetadata(
        ddes_version="1.0.0",
        datetime="now",
        experiment_id="experiment",
        run_id="run",
        ddes_type="assays",
    )
    assays.records.append(AssayRecord("001", "PSI.GREEN;ENV.YELLOW", "0;NA"))
    assays.records[0].probe_concentrations_nm = None  # type: ignore[assignment]
    assays.records.append(intensity.records[0])  # type: ignore[arg-type]
    write_assays(AssaysFile(assays.metadata, assays.records))
    write_assays(main)  # type: ignore[arg-type]
    constructed = DdesArchive(main, assays, [intensity])
    constructed.set_main(main)
    constructed.set_assays(assays)
    constructed.set_intensities([intensity])
    constructed.set_main(assays)  # type: ignore[arg-type]
    constructed.set_assays(main)  # type: ignore[arg-type]
    constructed.set_intensities([main])  # type: ignore[list-item]
    DdesArchive(main, assays, [main])  # type: ignore[list-item]
    constructed.write_path(path)
    archive = DdesArchive.read_path(path)
    DdesArchive.from_bytes(data)
    write_main(archive.main())
    write_assays(archive.assays())
    write_intensity(archive.intensity(0))
    files: list[IntensityFile] = archive.intensities()
    archive.set_intensities(files)
    return archive.to_bytes()
