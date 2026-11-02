library(ddes)
testthat::test_dir("r/tests/testthat", reporter = "summary", stop_on_failure = TRUE)
scratch <- tempfile("ddes-help-")
dir.create(scratch)
tryCatch({
  for (file in list.files("r/man", full.names = TRUE)) {
    rd <- tools::parse_Rd(file)
    tools::checkRd(rd)
    example <- file.path(scratch, paste0(basename(file), ".R"))
    tools::Rd2ex(rd, example)
    if (file.exists(example)) sys.source(example, envir = new.env())
  }
}, finally = unlink(scratch, recursive = TRUE))
