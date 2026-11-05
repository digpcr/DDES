library(ddes)

roundtrip <- function(input, output = NULL) {
  dataset <- read_archive(input)
  dataset$main$metadata$experiment_id <- "edited"
  if (nrow(dataset$main$records)) {
    dataset$main$records$counts_positive[1] <- "42"
    dataset$main$records$threshold_if_threshold[1] <- NA_real_
  }
  if (nrow(dataset$assays$records)) {
    dataset$assays$records$probe_concentrations_nm[1] <- "125;NA;0"
  }
  if (length(dataset$intensities) && nrow(dataset$intensities[[1]]$records)) {
    dataset$intensities[[1]]$records$fi_endpoint[1] <- -0.5
  }

  stopifnot(identical(read_main(write_main(dataset$main)), dataset$main))
  stopifnot(identical(read_assays(write_assays(dataset$assays)), dataset$assays))
  for (intensity in dataset$intensities) {
    stopifnot(identical(read_intensity(write_intensity(intensity)), intensity))
  }
  stopifnot(identical(archive_from_raw(archive_to_raw(dataset)), dataset))
  if (!is.null(output)) {
    write_archive(dataset, output)
    stopifnot(identical(read_archive(output), dataset))
  }
  message("Verified edited components and archive round trip.")
  invisible(dataset)
}

if (sys.nframe() == 0L) {
  args <- commandArgs(trailingOnly = TRUE)
  if (!length(args) || length(args) > 2L) stop("Usage: Rscript roundtrip.R INPUT [OUTPUT]")
  roundtrip(args[[1]], if (length(args) == 2L) args[[2]] else NULL)
}
