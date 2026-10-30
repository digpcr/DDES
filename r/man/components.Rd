\name{components}
\alias{read_main}
\alias{write_main}
\alias{read_assays}
\alias{write_assays}
\alias{read_intensity}
\alias{write_intensity}
\title{Read and Write DDES Components}
\description{Read, edit, and write complete DDES v1.0.0 TSV components, including metadata and column headers. Scientific results are preserved without recalculation.}
\usage{
read_main(bytes)
write_main(file)
read_assays(bytes)
write_assays(file)
read_intensity(bytes)
write_intensity(file)
}
\arguments{
\item{bytes}{A raw vector containing a complete UTF-8 TSV, not a filename.}
\item{file}{A list with exactly \code{metadata} (a named list of scalar strings) and \code{records} (a data frame). See \link{ddes-data} for all required fields.}
}
\value{Readers return editable component lists. Writers return raw vectors and do not modify their input.}
\details{
Field names use snake case. Every field remains required, including when its value is missing.
Main counts use decimal character columns from \code{"0"} through \code{"18446744073709551615"}, or \code{NA_character_}.
Measurements use numeric columns; optional main measurements accept \code{NA_real_} or \code{NA_integer_}.
\code{NaN} and infinities are invalid, not missing values. Endpoint intensity cannot be missing.
Identifiers must be character values, not factors or numbers. Probe concentrations use semicolon-separated strings; \code{"NA"} denotes a missing whole list or individual entry.

Readers and writers signal R errors for malformed data, unsupported fields, and invalid numbers. Writers normalize text formatting, including line endings and numeric spelling. They reject missing, extra, and duplicate fields, unequal column lengths, and invalid metadata. Data-frame row names are not stored in DDES.
}
\examples{
intensity <- list(
  metadata = list(ddes_version = "1.0.0", datetime = "202601011200",
    experiment_id = "example", run_id = "1", ddes_type = "intensity",
    well_id = "A1", assay_id = "001"),
  records = data.frame(partition_id = c("01", "02"),
    channel_id = c("FAM", "FAM"), fi_endpoint = c(-0.5, 120.25))
)
bytes <- write_intensity(intensity)
stopifnot(identical(read_intensity(bytes), intensity))
}
\seealso{\code{\link{read_archive}}, \link{ddes-data}}
