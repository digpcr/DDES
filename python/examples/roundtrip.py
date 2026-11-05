"""Edit all component types and optionally save a new archive."""

import sys
from pathlib import Path

from ddes import (
    DdesArchive,
    read_assays,
    read_intensity,
    read_main,
    write_assays,
    write_intensity,
    write_main,
)


def main(path: Path, output: Path | None = None) -> None:
    archive = DdesArchive.read_path(path)
    summary = archive.main()
    if summary.records:
        summary.records[0].counts_positive = 2**64 - 1
        summary.records[0].threshold_if_threshold = None
    if read_main(write_main(summary)) != summary:
        raise RuntimeError("Main TSV round trip changed component data")
    assays = archive.assays()
    if assays.records:
        assays.records[0].probe_concentrations_nm = "NA"
    if read_assays(write_assays(assays)) != assays:
        raise RuntimeError("Assays TSV round trip changed component data")
    intensities = archive.intensities()
    for intensity in intensities:
        if intensity.records:
            intensity.records[0].fi_endpoint = -12.75
        if read_intensity(write_intensity(intensity)) != intensity:
            raise RuntimeError("Intensity TSV round trip changed component data")
    # Copy edited snapshots back into the archive explicitly.
    archive.set_main(summary)
    archive.set_assays(assays)
    archive.set_intensities(intensities)
    # The same components can also construct an independent archive.
    created = DdesArchive(summary, assays, intensities)
    restored = DdesArchive.from_bytes(archive.to_bytes())
    if not restored.main() == summary == created.main():
        raise RuntimeError("Archive round trip changed main data")
    if not restored.assays() == assays == created.assays():
        raise RuntimeError("Archive round trip changed assays data")
    if not restored.intensities() == intensities == created.intensities():
        raise RuntimeError("Archive round trip changed intensity data or order")
    if output is not None:
        archive.write_path(output)  # Refuses to overwrite an existing file.
        saved = DdesArchive.read_path(output)
        if (
            saved.main() != summary
            or saved.assays() != assays
            or saved.intensities() != intensities
        ):
            raise RuntimeError("Saved archive round trip changed component data")
    print(
        f"Verified edited archive round trip: {len(summary.records)} main rows, "
        f"{len(assays.records)} assays, "
        f"{archive.intensity_count} intensity files"
    )


if __name__ == "__main__":
    if len(sys.argv) not in (2, 3):
        raise SystemExit(
            "usage: python examples/roundtrip.py <tsv-directory-or-archive> [new-output.ddes]"
        )
    main(Path(sys.argv[1]), Path(sys.argv[2]) if len(sys.argv) == 3 else None)
