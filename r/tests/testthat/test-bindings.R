library(ddes)

# Deliberately asymmetric values, exact large counts, and IDs with leading zeros.
dataset <- function() {
  meta <- list(ddes_version = "1.0.0", datetime = "202601011200",
               experiment_id = "exp", run_id = "run", ddes_type = "main")
  main <- list(metadata = c(meta, list(instrument = "test", plate_type = "96",
                                      instrument_software_version = "1")),
    records = data.frame(well_id = c("A1", "A1"), sample_id = c("001", "é"),
      assay_id = c("01", "01"), target_id = c("01", "02"),
      counts_positive = c("9007199254740993", "18446744073709551615"),
      counts_negative = c("0", NA_character_),
      concentration_reaction_cp_ul = c(0, NA_real_), threshold_if_threshold = c(-1.5, NA_real_)))
  meta$ddes_type <- "assays"
  assays <- list(metadata = meta, records = data.frame(assay_id = "01",
    probe_combination = "FAM;HEX", probe_concentrations_nm = "125;NA;0"))
  meta$ddes_type <- "intensity"
  intensity <- list(metadata = c(meta, list(well_id = "A1", assay_id = "01")),
    records = data.frame(partition_id = c("001", "002", "001", "002"),
      channel_id = c("FAM", "FAM", "HEX", "HEX"), fi_endpoint = c(-0.5, 2, 8.25, 100)))
  list(main = main, assays = assays, intensities = list(intensity))
}

test_that("all components and raw archives preserve values and independent edits", {
  x <- dataset()
  expect_identical(read_main(write_main(x$main)), x$main)
  expect_identical(read_assays(write_assays(x$assays)), x$assays)
  expect_identical(read_intensity(write_intensity(x$intensities[[1]])), x$intensities[[1]])
  expect_identical(archive_from_raw(archive_to_raw(x)), x)
  edited <- x
  edited$main$records$counts_positive[1] <- "42"
  edited$assays$records$probe_concentrations_nm <- "NA"
  edited$intensities[[1]]$records$fi_endpoint[1] <- -2
  expect_identical(archive_from_raw(archive_to_raw(edited)), edited)
  expect_identical(x, dataset())
  edited$main$records <- edited$main$records[0, , drop = FALSE]
  edited$assays$records <- edited$assays$records[0, , drop = FALSE]
  edited$intensities[[1]]$records <- edited$intensities[[1]]$records[0, , drop = FALSE]
  expect_identical(archive_from_raw(archive_to_raw(edited)), edited)
  edited$intensities <- list()
  expect_identical(archive_from_raw(archive_to_raw(edited)), edited)
})

test_that("counts never silently round and missing numbers differ from NaN", {
  x <- dataset()$main
  for (invalid in list(c(1, 2), c(1L, 2L), c(TRUE, FALSE), c("-1", "0"),
                       c("18446744073709551616", "0"), c("1.5", "0"), c("NA", "0"))) {
    y <- x
    y$records$counts_positive <- invalid
    expect_error(write_main(y), "counts_positive")
  }
  for (field in c("concentration_reaction_cp_ul", "threshold_if_threshold")) {
    for (invalid in c(NaN, Inf, -Inf)) {
      y <- x
      y$records[[field]][1] <- invalid
      expect_error(write_main(y), "finite")
    }
  }
  y <- x
  y$records$concentration_reaction_cp_ul[1] <- -0.5
  expect_error(write_main(y), "nonnegative")
  y$records$concentration_reaction_cp_ul <- c(0L, NA_integer_)
  expect_identical(read_main(write_main(y))$records$concentration_reaction_cp_ul, c(0, NA_real_))
  intensity <- dataset()$intensities[[1]]
  for (invalid in c(NA_real_, NaN, Inf, -Inf)) {
    intensity$records$fi_endpoint[1] <- invalid
    expect_error(write_intensity(intensity), "(NA is not allowed|finite)")
  }
})

test_that("invalid shapes and unsupported columns are rejected without changing inputs", {
  x <- dataset()
  bad <- x$main
  bad$records$extra <- 1
  expect_error(write_main(bad), "exactly")
  bad <- x$main
  names(bad$records)[2] <- "well_id"
  expect_error(write_main(bad), "exactly")
  bad <- x$main
  bad$metadata$extra <- "unused"
  expect_error(write_main(bad), "exactly")
  bad <- x$main
  bad$metadata$instrument <- NA_character_
  expect_error(write_main(bad), "NA")
  bad <- x$main
  bad$records$sample_id <- factor(bad$records$sample_id)
  expect_error(write_main(bad), "character")
  bad <- x$main
  bad$records <- structure(list(well_id = "A1", sample_id = c("1", "2"),
    assay_id = "1", target_id = "1", counts_positive = "1", counts_negative = "0",
    concentration_reaction_cp_ul = 0, threshold_if_threshold = 0), class = "data.frame", row.names = 1:2)
  expect_error(write_main(bad), "equal length")
  x$intensities[[2]] <- x$intensities[[1]]
  expect_error(archive_to_raw(x), "duplicate")
  x$intensities[[2]]$records$fi_endpoint[1] <- Inf
  before <- x
  expect_error(archive_to_raw(x), "finite")
  expect_identical(x, before)
  expect_error(read_main(charToRaw("garbage")))
  expect_error(archive_from_raw(charToRaw("not a zip")))
  expect_error(read_intensity("not raw"))
})

test_that("paths refuse overwrite and validation precedes file creation", {
  path <- tempfile(fileext = ".ddes")
  on.exit(unlink(path))
  x <- dataset()
  expect_invisible(write_archive(x, path))
  expect_identical(read_archive(path), x)
  original <- readBin(path, "raw", n = file.info(path)$size)
  expect_error(write_archive(x, path))
  expect_identical(readBin(path, "raw", n = file.info(path)$size), original)
  unlink(path)
  x$main$metadata$experiment_id <- "../unsafe"
  expect_error(write_archive(x, path), "filename")
  expect_false(file.exists(path))
  expect_error(read_archive(path))
  expect_error(read_archive(NA_character_), "path")
})

test_that("non-UTF8 R strings are translated and list spelling is preserved", {
  x <- dataset()
  x$main$records$sample_id[1] <- iconv("café", to = "latin1")
  expect_identical(read_main(write_main(x$main))$records$sample_id[1], "café")
  for (value in c("NA", "125;NA;0", "1e2;0.50")) {
    x$assays$records$probe_concentrations_nm <- value
    expect_identical(read_assays(write_assays(x$assays))$records$probe_concentrations_nm, value)
  }
  for (value in c("-1", "Inf", "NaN", NA_character_)) {
    x$assays$records$probe_concentrations_nm <- value
    expect_error(write_assays(x$assays))
  }
})
