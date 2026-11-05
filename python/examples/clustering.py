"""Cluster partitions by endpoint intensities, independently for each well.

Uses all channels without scaling. Labels are arbitrary, local to each well,
and are not positive/negative calls or target assignments. No DDES values are
changed. Every partition must have exactly one measurement per channel.
"""

import argparse
import csv
import sys
from pathlib import Path

from sklearn.cluster import KMeans  # type: ignore[import-untyped]

from ddes import DdesArchive, IntensityFile


def cluster_intensities(file: IntensityFile, n_clusters: int) -> list[tuple[str, int]]:
    """Return (partition ID, cluster label) pairs in partition-ID order."""
    if n_clusters < 1:
        raise ValueError("Cluster count must be positive")

    # TSV rows may be grouped by channel rather than by partition.
    partitions: dict[str, dict[str, float]] = {}
    for record in file.records:
        channels = partitions.setdefault(record.partition_id, {})
        if record.channel_id in channels:
            raise ValueError(
                f"Duplicate measurement: {record.partition_id!r}, {record.channel_id!r}"
            )
        channels[record.channel_id] = record.fi_endpoint

    channel_ids = sorted({channel for row in partitions.values() for channel in row})
    partition_ids = sorted(partitions)
    measurements: list[list[float]] = []
    for partition_id in partition_ids:
        row = partitions[partition_id]
        if len(row) != len(channel_ids):
            raise ValueError(
                f"Partition {partition_id!r} is missing channel measurements"
            )
        measurements.append([row[channel] for channel in channel_ids])

    if len({tuple(row) for row in measurements}) < n_clusters:
        raise ValueError("Need at least as many distinct intensity vectors as clusters")
    labels = KMeans(n_clusters=n_clusters, random_state=0, n_init=10).fit_predict(
        measurements
    )
    return [(partition, int(label)) for partition, label in zip(partition_ids, labels)]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", type=Path, help="TSV directory or ZIP/.ddes archive")
    parser.add_argument(
        "--clusters", type=int, default=2, help="Clusters per well (default: 2)"
    )
    args = parser.parse_args()
    if args.clusters < 1:
        parser.error("--clusters must be positive")
    archive = DdesArchive.read_path(args.path)
    if archive.intensity_count == 0:
        parser.error("Archive has no intensity files")

    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(["well_id", "assay_id", "partition_id", "cluster"])
    for file in archive.intensities():
        try:
            assignments = cluster_intensities(file, args.clusters)
        except ValueError as error:
            parser.error(f"Well {file.metadata.well_id!r}: {error}")
        for partition_id, label in assignments:
            writer.writerow(
                [file.metadata.well_id, file.metadata.assay_id, partition_id, label]
            )


if __name__ == "__main__":
    main()
