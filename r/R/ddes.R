# Convert R's native/Latin-1 strings before extendr borrows UTF-8 text.
.ddes_utf8 <- function(x) {
  if (is.character(x)) {
    if (any(Encoding(x) == "bytes")) stop("byte-encoded strings are not supported", call. = FALSE)
    x <- enc2utf8(x)
    if (any(!validUTF8(x))) stop("invalid UTF-8 string", call. = FALSE)
  } else if (is.list(x)) {
    attrs <- attributes(x)
    x <- lapply(unclass(x), .ddes_utf8)
    attributes(x) <- attrs
  }
  if (!is.null(names(x))) names(x) <- .ddes_utf8(names(x))
  x
}

.ddes_call <- function(symbol, ...) {
  value <- .Call(symbol, ...)
  if (inherits(value, "extendr_error")) stop(value$value, call. = FALSE)
  value
}

.ddes_path <- function(path) {
  if (!is.character(path) || length(path) != 1L || is.na(path)) {
    stop("path must be one nonmissing character string", call. = FALSE)
  }
  .ddes_utf8(path.expand(path))
}

read_main <- function(bytes) .ddes_call(wrap__read_main, bytes)
write_main <- function(file) .ddes_call(wrap__write_main, .ddes_utf8(file))
read_assays <- function(bytes) .ddes_call(wrap__read_assays, bytes)
write_assays <- function(file) .ddes_call(wrap__write_assays, .ddes_utf8(file))
read_intensity <- function(bytes) .ddes_call(wrap__read_intensity, bytes)
write_intensity <- function(file) .ddes_call(wrap__write_intensity, .ddes_utf8(file))
read_archive <- function(path) .ddes_call(wrap__read_archive, .ddes_path(path))
write_archive <- function(dataset, path) {
  invisible(.ddes_call(wrap__write_archive, .ddes_utf8(dataset), .ddes_path(path)))
}
archive_from_raw <- function(bytes) .ddes_call(wrap__archive_from_raw, bytes)
archive_to_raw <- function(dataset) .ddes_call(wrap__archive_to_raw, .ddes_utf8(dataset))
