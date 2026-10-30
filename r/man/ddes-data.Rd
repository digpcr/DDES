\name{ddes-data}
\alias{ddes-data}
\title{DDES Metadata and Record Fields}
\description{Components are lists containing a metadata list and a records data frame. Fields and columns may be reordered, but may not be omitted, duplicated, or extended. Construct new components with ordinary lists and data frames.}
\section{Metadata}{All values are nonmissing scalar character strings.
\describe{
\item{ddes_version}{Version spelling, such as \code{"1.0"} or \code{"1.0.0"}.}
\item{datetime}{Export timestamp as reported; no date grammar is enforced.}
\item{experiment_id, run_id}{Experiment and run identifiers. Main metadata determines the archive's shared filename prefix.}
\item{ddes_type}{\code{"main"}, \code{"assays"} (also accepts \code{"assay"}), or \code{"intensity"}, according to component type.}
\item{instrument, plate_type, instrument_software_version}{Required only in main metadata: reported instrument, plate, and software version.}
\item{well_id, assay_id}{Required only in intensity metadata: well and assay for these measurements.}
}}
\section{Main records}{
\describe{
\item{well_id, sample_id, assay_id, target_id}{Character identifiers for the reaction and target; leading zeros and Unicode are preserved. No missing values.}
\item{counts_positive, counts_negative}{Reported positive/negative partition counts. Decimal character values in the unsigned 64-bit range, or \code{NA_character_}; missing does not mean zero.}
\item{concentration_reaction_cp_ul}{Finite nonnegative numeric reaction concentration in copies per microlitre, or numeric NA.}
\item{threshold_if_threshold}{Finite numeric threshold, or numeric NA when unavailable/inapplicable. Negative thresholds are allowed.}
}}
\section{Assay records}{
\describe{
\item{assay_id}{Nonmissing character assay identifier.}
\item{probe_combination}{Nonmissing character probe/target and channel combinations, retained as an unparsed list string.}
\item{probe_concentrations_nm}{Semicolon-separated finite nonnegative concentrations in nanomolar. \code{"NA"} means the entire list is missing; \code{"125;NA;0"} contains a missing entry. Spelling is preserved and list length is not checked against probes. Use the literal string \code{"NA"}, not R's missing string.}
}}
\section{Intensity records}{
\describe{
\item{partition_id, channel_id}{Nonmissing character partition and detection-channel identifiers.}
\item{fi_endpoint}{Required finite numeric endpoint fluorescence. Zero and negative processed values are allowed.}
}}
\section{Editing}{
Replace values, append or remove rows, or replace entire components with normal R operations. Keep zero-row records as data frames with typed columns, using \code{x$records[0, , drop = FALSE]}. Optional numbers use typed NA values, not \code{NULL}. Factors, logical measurements, and classed numeric vectors are not accepted. Scientific validation happens when reading or writing; temporary invalid edits are possible in R.
}
