\name{archive}
\alias{read_archive}
\alias{write_archive}
\alias{archive_from_raw}
\alias{archive_to_raw}
\title{Read and Write DDES Archives}
\description{Read and edit complete DDES datasets as R lists, and write ZIP archives without recalculating reported scientific results.}
\usage{
read_archive(path)
write_archive(dataset, path)
archive_from_raw(bytes)
archive_to_raw(dataset)
}
\arguments{
\item{path}{One nonmissing character path. Readers accept a flat TSV directory or ZIP archive independent of extension. Writers create a new ZIP file; existing destinations are never overwritten. A leading tilde is expanded.}
\item{dataset}{A list with exactly \code{main}, \code{assays}, and \code{intensities}. Main and assays are component lists; intensities is a list of components, or \code{list()} when absent. See \link{ddes-data}.}
\item{bytes}{A raw vector containing a ZIP archive.}
}
\value{Readers return editable dataset lists. \code{archive_to_raw} returns a raw vector. \code{write_archive} returns \code{NULL} invisibly.}
\details{
All operations load complete datasets into memory, with no input size limits. Returned values have normal R copy-on-modify behavior; there are no handles to close. Writers do not modify their input.

Reads sort intensity components by well ID. Writes preserve supplied order. Filenames use the main metadata's experiment ID, run ID, and datetime; intensity filenames also include their well ID. IDs in assays and intensity headers are preserved but do not determine the shared filename prefix.

Malformed data, I/O failures, unsafe filenames, duplicate components or output filenames, and missing main/assays components signal R errors. Validation finishes before a path destination is created. An I/O failure can leave a partial file. Cross-file scientific consistency is not checked. Text and ZIP formatting may normalize; byte-for-byte round trips are not promised.
}
\seealso{\code{\link{read_main}}, \code{\link{read_assays}}, \code{\link{read_intensity}}, \link{ddes-data}}
