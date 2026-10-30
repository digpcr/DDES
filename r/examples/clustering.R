library(ddes)

# Align each partition's channels before clustering. IDs are never made numeric.
cluster_well <- function(intensity, centers) {
  if (length(centers) != 1L || !is.numeric(centers) || is.na(centers) ||
      !is.finite(centers) || centers < 1 || centers != floor(centers)) {
    stop("centers must be one positive integer")
  }
  rows <- intensity$records
  if (!nrow(rows) || anyNA(rows) || any(!is.finite(rows$fi_endpoint))) {
    stop("clustering needs nonempty, finite measurements")
  }
  if (anyDuplicated(rows[c("partition_id", "channel_id")])) {
    stop("duplicate partition/channel measurement")
  }
  partitions <- unique(rows$partition_id)
  channels <- unique(rows$channel_id)
  values <- matrix(NA_real_, nrow = length(partitions), ncol = length(channels))
  values[cbind(match(rows$partition_id, partitions), match(rows$channel_id, channels))] <- rows$fi_endpoint
  if (anyNA(values)) stop("each partition must have one measurement per channel")
  if (centers > nrow(unique(values))) stop("too few distinct measurement vectors")
  fit <- stats::kmeans(values, centers = centers, nstart = 10, iter.max = 100)
  if (!is.null(fit$ifault) && fit$ifault != 0L) stop("k-means did not converge")
  data.frame(
    well_id = intensity$metadata$well_id, assay_id = intensity$metadata$assay_id,
    partition_id = partitions, cluster = fit$cluster, row.names = NULL
  )
}

if (sys.nframe() == 0L) {
  args <- commandArgs(trailingOnly = TRUE)
  if (length(args) != 2L) stop("Usage: Rscript clustering.R INPUT CLUSTERS")
  dataset <- read_archive(args[[1]])
  centers <- as.numeric(args[[2]])
  set.seed(42)
  assignments <- lapply(dataset$intensities, cluster_well, centers = centers)
  output <- if (length(assignments)) do.call(rbind, assignments) else {
    data.frame(well_id = character(), assay_id = character(),
               partition_id = character(), cluster = integer())
  }
  utils::write.csv(output, stdout(), row.names = FALSE)
}
