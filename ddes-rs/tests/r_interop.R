# Optional independent compatibility check; no R dependency in cargo test.
# Usage: Rscript tests/r_interop.R <source-tsv-dir> <output.zip> [reference-repo]
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) %in% c(2L, 3L))
source_dir <- normalizePath(args[1], mustWork = TRUE)
archive <- normalizePath(args[2], mustWork = TRUE)
reference <- if (length(args) == 3L) normalizePath(args[3], mustWork = TRUE) else NULL

check <- function() {
  root <- tempfile("ddes-r-interop-")
  dir.create(file.path(root, "demo_data", "generated"), recursive = TRUE)
  on.exit(unlink(root, recursive = TRUE), add = TRUE)
  output_dir <- file.path(root, "demo_data", "generated")
  unzip(archive, exdir = output_dir)
  originals <- list.files(source_dir, pattern = "\\.tsv$", full.names = TRUE)
  outputs <- list.files(output_dir, pattern = "\\.tsv$", full.names = TRUE)
  stopifnot(length(originals) > 0L, length(originals) == length(outputs))
  role <- function(path) sub("^.*_(main|assays?|intensity_.*)\\.tsv$", "\\1", basename(path))
  read <- function(path) read.table(path, sep = "\t", stringsAsFactors = FALSE,
                                  comment.char = "#", header = TRUE)
  for (original in originals) {
    output <- outputs[vapply(outputs, role, "") == role(original)]
    stopifnot(length(output) == 1L)
    # These are the reference reader's table options, including its type inference.
    stopifnot(isTRUE(all.equal(read(original), read(output), tolerance = 0)))
    header_lines <- if (role(original) == "main") 10L else if (role(original) %in% c("assay", "assays")) 8L else 9L
    stopifnot(identical(readLines(original, n = header_lines), readLines(output, n = header_lines)))
  }
  cat("R read.table: all", length(outputs), "tables and fixed metadata headers match\n")

  if (!is.null(reference)) {
    # Evaluate only the pinned reader function, not app startup/server code.
    # The only stubs are Shiny's progress indicators; parsing is upstream code.
    env <- new.env(parent = globalenv())
    # Prebind outputs so upstream's <<- assignments stay in this environment.
    env$ddes_import_main <- env$ddes_import_assay <- NULL
    env$intensity_files_data_list <- env$intensity_files_header_list <- NULL
    env$str_split <- stringr::str_split
    env$withProgress <- function(message, value, expr) eval.parent(substitute(expr))
    env$incProgress <- function(...) invisible(NULL)
    expressions <- parse(file.path(reference, "global.R"))
    definition <- Filter(function(expr) is.call(expr) && identical(expr[[1]], as.name("<-")) &&
                           identical(expr[[2]], as.name("retrieve_demo_dataset_ddes")), expressions)
    stopifnot(length(definition) == 1L)
    eval(definition[[1]], envir = env)
    old <- setwd(root)
    on.exit(setwd(old), add = TRUE)
    env$retrieve_demo_dataset_ddes("generated")
    main <- originals[vapply(originals, role, "") == "main"]
    assays <- originals[vapply(originals, role, "") %in% c("assay", "assays")]
    expected <- read(main)
    stopifnot(isTRUE(all.equal(env$ddes_import_main[names(expected)], expected, tolerance = 0)))
    expected <- read(assays)
    stopifnot(isTRUE(all.equal(env$ddes_import_assay[names(expected)], expected, tolerance = 0)))
    intensity_files <- originals[grepl("^intensity_", vapply(originals, role, ""))]
    for (original in intensity_files) {
      well <- sub("intensity_", "", role(original), fixed = TRUE)
      index <- which(vapply(env$intensity_files_header_list, function(h) h[7] == paste0("#well_ID\t", well), TRUE))
      stopifnot(length(index) == 1L)
      expected <- read(original)
      expected <- expected[!is.na(expected$FI_endpoint), ]
      stopifnot(isTRUE(all.equal(env$intensity_files_data_list[[index]], expected, tolerance = 0)))
    }
    cat("Upstream retrieve_demo_dataset_ddes: main, assays, and", length(intensity_files), "wells match\n")
  }
}
check()
