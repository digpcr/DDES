from os import PathLike
from typing import final

__all__ = [
    "AssayRecord",
    "AssaysFile",
    "AssaysMetadata",
    "DdesArchive",
    "IntensityFile",
    "IntensityMetadata",
    "IntensityRecord",
    "MainFile",
    "MainMetadata",
    "MainRecord",
    "read_assays",
    "read_intensity",
    "read_main",
    "write_assays",
    "write_intensity",
    "write_main",
]

@final
class AssaysMetadata:
    ddes_version: str
    datetime: str
    experiment_id: str
    run_id: str
    ddes_type: str
    def __new__(
        cls,
        *,
        ddes_version: str,
        datetime: str,
        experiment_id: str,
        run_id: str,
        ddes_type: str,
    ) -> AssaysMetadata: ...

@final
class AssayRecord:
    assay_id: str
    probe_combination: str
    probe_concentrations_nm: str
    def __new__(
        cls, assay_id: str, probe_combination: str, probe_concentrations_nm: str
    ) -> AssayRecord: ...

@final
class AssaysFile:
    metadata: AssaysMetadata
    records: list[AssayRecord]
    def __new__(
        cls, metadata: AssaysMetadata, records: list[AssayRecord]
    ) -> AssaysFile: ...

@final
class MainMetadata:
    ddes_version: str
    datetime: str
    experiment_id: str
    run_id: str
    ddes_type: str
    instrument: str
    plate_type: str
    instrument_software_version: str
    def __new__(
        cls,
        *,
        ddes_version: str,
        datetime: str,
        experiment_id: str,
        run_id: str,
        ddes_type: str,
        instrument: str,
        plate_type: str,
        instrument_software_version: str,
    ) -> MainMetadata: ...

@final
class MainRecord:
    well_id: str
    sample_id: str
    assay_id: str
    target_id: str
    counts_positive: int | None
    counts_negative: int | None
    concentration_reaction_cp_ul: float | None
    threshold_if_threshold: float | None
    def __new__(
        cls,
        *,
        well_id: str,
        sample_id: str,
        assay_id: str,
        target_id: str,
        counts_positive: int | None,
        counts_negative: int | None,
        concentration_reaction_cp_ul: float | None,
        threshold_if_threshold: float | None,
    ) -> MainRecord: ...

@final
class MainFile:
    metadata: MainMetadata
    records: list[MainRecord]
    def __new__(cls, metadata: MainMetadata, records: list[MainRecord]) -> MainFile: ...

@final
class IntensityMetadata:
    ddes_version: str
    datetime: str
    experiment_id: str
    run_id: str
    ddes_type: str
    well_id: str
    assay_id: str
    def __new__(
        cls,
        *,
        ddes_version: str,
        datetime: str,
        experiment_id: str,
        run_id: str,
        ddes_type: str,
        well_id: str,
        assay_id: str,
    ) -> IntensityMetadata: ...

@final
class IntensityRecord:
    partition_id: str
    channel_id: str
    fi_endpoint: float
    def __new__(
        cls, partition_id: str, channel_id: str, fi_endpoint: float
    ) -> IntensityRecord: ...

@final
class IntensityFile:
    metadata: IntensityMetadata
    records: list[IntensityRecord]
    def __new__(
        cls, metadata: IntensityMetadata, records: list[IntensityRecord]
    ) -> IntensityFile: ...

def read_main(data: bytes) -> MainFile: ...
def write_main(file: MainFile) -> bytes: ...
def read_intensity(data: bytes) -> IntensityFile: ...
def write_intensity(file: IntensityFile) -> bytes: ...
def read_assays(data: bytes) -> AssaysFile: ...
def write_assays(file: AssaysFile) -> bytes: ...
@final
class DdesArchive:
    def __new__(
        cls, main: MainFile, assays: AssaysFile, intensities: list[IntensityFile]
    ) -> DdesArchive: ...
    @staticmethod
    def read_path(path: str | PathLike[str]) -> DdesArchive: ...
    @staticmethod
    def from_bytes(data: bytes) -> DdesArchive: ...
    def to_bytes(self) -> bytes: ...
    def write_path(self, path: str | PathLike[str]) -> None: ...
    @property
    def intensity_count(self) -> int: ...
    def main(self) -> MainFile: ...
    def assays(self) -> AssaysFile: ...
    def intensity(self, index: int) -> IntensityFile: ...
    def intensities(self) -> list[IntensityFile]: ...
    def set_main(self, file: MainFile) -> None: ...
    def set_assays(self, file: AssaysFile) -> None: ...
    def set_intensities(self, files: list[IntensityFile]) -> None: ...
