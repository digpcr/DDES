# Run from the repository root: Rscript r/examples/shiny/tests.R
library(testthat)
library(ddes)
source("r/examples/shiny/annotations.R")
source("r/examples/shiny/converters.R")
source("r/examples/shiny/session.R")
fixtures <- normalizePath("ddes-rs/tests/fixtures")

# Use the widget's real column metadata and row representation for browser edits.
hot_input <- function(rows) {
  widget <- rhandsontable::rhandsontable(rows, rowHeaders = NULL)
  list(data = jsonlite::fromJSON(widget$x$data, simplifyVector = FALSE),
       changes = list(event = "afterChange"), params = widget$x)
}

test_that("Shiny edits and downloads use bindings without changing reported values", {
  shiny::testServer(ddes_server(fixtures), {
    session$setInputs(start_demo = 1)
    original <- state$dataset
    expect_identical(original, read_archive(fixtures))
    session$setInputs(experiment_id = "edited", run_id = "run_2", instrument = "test",
      software = "2", plate = "96", save_run = 1)
    expect_identical(state$dataset$main$metadata$experiment_id, "edited")
    expect_identical(state$dataset$assays$metadata$run_id, "run_2")
    expect_identical(state$dataset$intensities[[1]]$metadata$run_id, "run_2")
    expect_identical(state$dataset$main$records, original$main$records)
    probes <- transform(state$probes, selected = FALSE)
    probes$target_id[1] <- "renamed_target"
    session$setInputs(probes = hot_input(probes), save_probes = 1)
    expect_true("renamed_target" %in% state$dataset$main$records$target_id)
    assay_rows <- transform(state$dataset$assays$records, selected = FALSE)
    assay_rows$assay_id[1] <- "renamed_assay"
    assay_rows$probe_concentrations_nm[1] <- "125;NA;0"
    session$setInputs(assays = hot_input(assay_rows), save_assays = 1)
    expect_true(all(state$dataset$main$records$assay_id == "renamed_assay"))
    expect_identical(state$dataset$intensities[[1]]$metadata$assay_id, "renamed_assay")
    samples <- transform(sample_table(state$dataset), selected = FALSE)
    samples$sample_id[1] <- "0001"
    session$setInputs(samples = hot_input(samples), save_samples = 1)
    expect_true(all(state$dataset$main$records$sample_id == "0001"))
    before <- state$dataset
    assay_rows$probe_concentrations_nm[1] <- "-1"
    session$setInputs(assays = hot_input(assay_rows), save_assays = 2)
    expect_identical(state$dataset, before)
    expect_match(state$message, "nonnegative")
    session$setInputs(prepare_download = 1)
    restored <- archive_from_raw(state$download)
    expect_identical(restored, before)
    expect_identical(restored$main$records$counts_positive, original$main$records$counts_positive)
    expect_identical(restored$intensities[[1]]$records, original$intensities[[1]]$records)
    session$setInputs(save_run = 2)
    expect_null(state$download)

    assays <- transform(state$dataset$assays$records, selected = TRUE)
    session$setInputs(assays = hot_input(assays), delete_assay = 1)
    expect_match(state$message, "still uses")
    expect_identical(state$dataset$assays$records, before$assays$records)
    probes <- transform(state$probes, selected = seq_len(nrow(state$probes)) == 1L)
    session$setInputs(probes = hot_input(probes), delete_probe = 1)
    expect_match(state$message, "still uses")
    session$setInputs(replace_assay_probes = 1)
    expect_identical(state$dataset$main$records$target_id, "renamed_target")
    expect_identical(state$dataset$assays$records$probe_concentrations_nm, "NA")
    expect_true(all(is.na(state$dataset$main$records[c("counts_positive", "counts_negative",
      "concentration_reaction_cp_ul", "threshold_if_threshold")])))
    expect_identical(state$dataset$intensities[[1]]$records, original$intensities[[1]]$records)

    session$setInputs(probe_target = "unused", probe_channel = "GREEN", add_probe = 1)
    probes <- transform(state$probes, selected = target_id == "unused")
    session$setInputs(probes = hot_input(probes), assay_name = "unused_assay", add_assay = 1)
    expect_identical(tail(state$dataset$assays$records$probe_combination, 1), "unused.GREEN")
    assays <- transform(state$dataset$assays$records, selected = assay_id == "unused_assay")
    session$setInputs(assays = hot_input(assays), delete_assay = 2)
    expect_false("unused_assay" %in% state$dataset$assays$records$assay_id)
    session$setInputs(delete_probe = 2)
    expect_false("unused" %in% state$probes$target_id)
  })
  shiny::testServer(ddes_server(fixtures), {
    expect_null(state$dataset)
    session$setInputs(start_demo = 1)
    expect_identical(state$dataset$main$metadata$experiment_id, "expID")
  })
})

test_that("assay assignment is exact by well and preserves unrelated results", {
  x <- read_archive(fixtures)
  second <- x$main$records
  second$well_id <- "A10"
  second$counts_positive <- "9007199254740993"
  x$main$records <- rbind(x$main$records, second)
  x$intensities[[2]] <- x$intensities[[1]]
  x$intensities[[2]]$metadata$well_id <- "A10"
  x$assays$records <- rbind(x$assays$records, data.frame(assay_id = "new",
    probe_combination = "new_target.GREEN", probe_concentrations_nm = "NA"))
  changed <- assign_assay(x, "A1", "new")
  a1 <- changed$main$records[changed$main$records$well_id == "A1", ]
  expect_identical(a1$target_id, "new_target")
  expect_true(all(is.na(a1[c("counts_positive", "counts_negative", "concentration_reaction_cp_ul", "threshold_if_threshold")])))
  expect_identical(changed$main$records$counts_positive[changed$main$records$well_id == "A10"], rep("9007199254740993", 3))
  expect_identical(changed$intensities[[1]]$metadata$assay_id, "new")
  expect_identical(changed$intensities[[2]], x$intensities[[2]])
  expect_identical(assign_assay(changed, "A1", "new"), changed)
  expect_identical(archive_from_raw(archive_to_raw(changed))$intensities[[1]]$records, x$intensities[[1]]$records)
})

test_that("QX200 conversion preserves counts and partition identity across channels", {
  dir <- withr::local_tempdir()
  results <- file.path(dir, "results.csv")
  amplitudes <- file.path(dir, "plate_A1_Amplitude.csv")
  writeLines(c("sep=,", "Well,Sample,TargetType,Target,Positives,Negatives,Concentration,Threshold",
    "A1,001,Ch1Unknown,alpha,9007199254740993,2,3.25,-1.5",
    "A1,001,Ch2Unknown,beta,0,18446744073709551615,NA,NA"), results)
  writeLines(c("Ch1,Ch2", "-0.5,100", "2.25,200"), amplitudes)
  upload <- data.frame(name = basename(amplitudes), datapath = amplitudes)
  x <- convert_qx200(results, "run.csv", upload)
  expect_identical(x$main$records$sample_id, c("001", "001"))
  expect_identical(x$main$records$counts_positive, c("9007199254740993", "0"))
  expect_identical(x$main$records$counts_negative, c("2", "18446744073709551615"))
  expect_identical(x$main$records$concentration_reaction_cp_ul, c(3.25, NA_real_))
  expect_identical(x$main$records$threshold_if_threshold, c(-1.5, NA_real_))
  expect_identical(x$intensities[[1]]$records$partition_id, c("1", "2", "1", "2"))
  expect_identical(x$intensities[[1]]$records$fi_endpoint, c(-0.5, 2.25, 100, 200))
  expect_identical(x$intensities[[1]]$metadata$assay_id, x$assays$records$assay_id[1])
  expect_equal(archive_from_raw(archive_to_raw(x)), x, ignore_attr = TRUE)
  expect_error(convert_qx200(results, "run.csv", rbind(upload, upload)), "exactly one")
  writeLines(c("Ch1,Ch2", "NA,100"), amplitudes)
  expect_error(convert_qx200(results, "run.csv", upload), "without missing")
})

test_that("QIAcuity conversion retains RFU data and maps multiple assays by exact ID", {
  dir <- withr::local_tempdir()
  results <- data.frame(Plate.ID = "run", Plate.type = "plate", Well = c("A1", "A10"),
    Sample.NTC.Control = c("001", "002"), Reaction.Mix = c("assay1", "assay2"),
    Target..Name. = c("alpha", "beta"), Partitions..Positive. = c("9007199254740993", "7"),
    Partitions..Negative. = c("9", "8"), Threshold = c("-1.5", "NA"))
  results[["Conc...cp.µL...dPCR.reaction."]] <- c("1.25", "NA")
  layout <- data.frame(Well = c("A1", "A10"), Reaction.mix.name = c("assay1", "assay2"))
  for (i in 1:5) {
    layout[[paste0("Channel.", i)]] <- if (i == 1) c("G", "Y") else "NA"
    layout[[paste0("Target.", i)]] <- if (i == 1) c("alpha", "beta") else "NA"
  }
  write.csv(results, file.path(dir, "results.csv"), row.names = FALSE)
  write.csv(layout, file.path(dir, "layout.csv"), row.names = FALSE)
  writeLines(c("RFU export", "Well,Partition,Channel,RFU", "A1,001,G,-0.5", "A10,009,Y,99.25"), file.path(dir, "plate_RFU.csv"))
  withr::with_dir(dir, utils::zip("rfu.zip", "plate_RFU.csv", flags = "-q"))
  x <- convert_qiacuity(file.path(dir, "results.csv"), file.path(dir, "layout.csv"), file.path(dir, "rfu.zip"))
  expect_identical(x$main$records$counts_positive, c("9007199254740993", "7"))
  expect_identical(x$assays$records$probe_combination, c("alpha.GREEN", "beta.YELLOW"))
  expect_identical(x$assays$records$probe_concentrations_nm, c("NA", "NA"))
  expect_identical(x$intensities[[1]]$records$partition_id, "001")
  expect_identical(x$intensities[[1]]$records$fi_endpoint, -0.5)
  expect_identical(x$intensities[[2]]$metadata$well_id, "A10")
  expect_identical(x$intensities[[2]]$metadata$assay_id, "assay2")
  expect_identical(x$intensities[[2]]$records$fi_endpoint, 99.25)
  expect_equal(archive_from_raw(archive_to_raw(x)), x, ignore_attr = TRUE)
  layout$Reaction.mix.name[2] <- "assay1"
  write.csv(layout, file.path(dir, "layout.csv"), row.names = FALSE)
  expect_error(convert_qiacuity(file.path(dir, "results.csv"), file.path(dir, "layout.csv"), file.path(dir, "rfu.zip")), "conflicting probe")
})
