"""Read, edit, and write Digital PCR Data Essentials Standard (DDES) data.

Use read_main/read_assays/read_intensity and their write counterparts for
complete TSV bytes. MainFile, AssaysFile, and IntensityFile contain editable
metadata and record objects. DdesArchive reads directories or ZIP archives and
copies components on construction, getters, and setters; explicitly save edited
snapshots back with set_main, set_assays, or set_intensities.

Numeric missing values use None, except assay concentration lists, which retain
their semicolon-separated text and NA markers. Scientific constraints are
checked on write. Operations are synchronous and in-memory, without size limits.

Use help(DdesArchive) or help(AssaysFile) for examples and the full API contract.
"""

from ._native import (
    AssayRecord as AssayRecord,
)
from ._native import (
    AssaysFile as AssaysFile,
)
from ._native import (
    AssaysMetadata as AssaysMetadata,
)
from ._native import (
    DdesArchive as DdesArchive,
)
from ._native import (
    IntensityFile as IntensityFile,
)
from ._native import (
    IntensityMetadata as IntensityMetadata,
)
from ._native import (
    IntensityRecord as IntensityRecord,
)
from ._native import (
    MainFile as MainFile,
)
from ._native import (
    MainMetadata as MainMetadata,
)
from ._native import (
    MainRecord as MainRecord,
)
from ._native import (
    read_assays as read_assays,
)
from ._native import (
    read_intensity as read_intensity,
)
from ._native import (
    read_main as read_main,
)
from ._native import (
    write_assays as write_assays,
)
from ._native import (
    write_intensity as write_intensity,
)
from ._native import (
    write_main as write_main,
)
