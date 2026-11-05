"""Run with the examples dependency group to exercise scikit-learn."""

import csv
import importlib.util
import io
import runpy
import subprocess
import sys
import unittest
from pathlib import Path

from ddes import IntensityRecord, read_intensity, write_intensity

ROOT = Path(__file__).resolve().parents[1]
EXAMPLE = ROOT / "examples" / "clustering.py"
FIXTURES = ROOT.parent / "ddes-rs" / "tests" / "fixtures"


@unittest.skipUnless(
    importlib.util.find_spec("sklearn"), "requires examples dependencies"
)
class ClusteringTest(unittest.TestCase):
    def setUp(self):
        self.cluster = runpy.run_path(str(EXAMPLE))["cluster_intensities"]
        self.file = read_intensity(
            (FIXTURES / "DDES_expID_run_202506301127_intensity_A1.tsv").read_bytes()
        )
        # Channel-major rows, with different partition orders per channel.
        # Only the second channel separates the two populations.
        self.file.records = [
            IntensityRecord(partition, channel, value)
            for partition, channel, value in [
                ("04", "BLUE", -2),
                ("001", "BLUE", -2),
                ("03", "BLUE", -1),
                ("010", "BLUE", -1),
                ("010", "GREEN", 103),
                ("03", "GREEN", -20),
                ("001", "GREEN", 100),
                ("04", "GREEN", -15),
            ]
        ]

    def test_multichannel_alignment_and_no_mutation(self):
        before = write_intensity(self.file)
        assignments = self.cluster(self.file, 2)
        self.assertEqual([p for p, _ in assignments], ["001", "010", "03", "04"])
        labels = dict(assignments)
        self.assertEqual(labels["001"], labels["010"])
        self.assertEqual(labels["03"], labels["04"])
        self.assertNotEqual(labels["001"], labels["03"])
        self.file.records.reverse()
        self.assertEqual(self.cluster(self.file, 2), assignments)
        self.file.records.reverse()
        self.assertEqual(write_intensity(self.file), before)
        self.assertEqual(set(dict(self.cluster(self.file, 1)).values()), {0})

    def test_invalid_measurement_matrices(self):
        for count in [0, -1, 5]:
            with self.subTest(count=count), self.assertRaises(ValueError):
                self.cluster(self.file, count)
        records = self.file.records[:]
        for invalid, message in [
            (records + [records[0]], "Duplicate"),
            (records[1:], "missing channel"),
            ([], "distinct intensity vectors"),
            (
                [IntensityRecord(str(i), "BLUE", 0) for i in range(3)],
                "distinct intensity vectors",
            ),
        ]:
            with self.subTest(message=message):
                self.file.records = invalid
                with self.assertRaisesRegex(ValueError, message):
                    self.cluster(self.file, 2)

    def test_cli_csv_preserves_partition_ids(self):
        result = subprocess.run(
            [sys.executable, str(EXAMPLE), str(FIXTURES), "--clusters", "2"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        rows = list(csv.DictReader(io.StringIO(result.stdout)))
        self.assertEqual([row["partition_id"] for row in rows], ["10", "11", "9"])
        self.assertEqual({row["well_id"] for row in rows}, {"A1"})
        self.assertEqual({row["assay_id"] for row in rows}, {"HIV Triplex"})
        self.assertEqual(rows[0]["cluster"], rows[1]["cluster"])
        self.assertNotEqual(rows[0]["cluster"], rows[2]["cluster"])
