# Probe and assay annotation operations; scientific measurements are not recalculated.

split_probes <- function(combinations) {
  tokens <- unique(as.character(unlist(strsplit(combinations, ";", fixed = TRUE))))
  if (anyNA(combinations) || any(!nzchar(combinations))) stop("Assays need a probe combination.")
  if (anyNA(tokens) || any(!grepl("^.+\\.[^.]+$", tokens))) {
    stop("Probe combinations must use target.channel entries separated by semicolons.")
  }
  data.frame(target_id = sub("\\.[^.]+$", "", tokens),
             channel_id = sub("^.*\\.", "", tokens), stringsAsFactors = FALSE)
}

probe_keys <- function(probes) paste(probes$target_id, probes$channel_id, sep = ".")

check_names <- function(values, label, unique = FALSE) {
  if (!is.character(values) || anyNA(values) || any(!nzchar(trimws(values))) ||
      any(grepl("[\t\r\n]", values))) stop(paste(label, "must be nonempty text without tabs or newlines."))
  if (unique && anyDuplicated(values)) stop(paste(label, "must be unique."))
}

sample_table <- function(dataset) {
  rows <- unique(dataset$main$records[c("well_id", "sample_id", "assay_id")])
  rownames(rows) <- NULL
  rows
}

rename_assays <- function(dataset, names) {
  old <- dataset$assays$records$assay_id
  check_names(names, "Assay IDs", unique = TRUE)
  if (length(names) != length(old)) stop("Save assay names without adding or removing rows.")
  replace_ids <- function(ids) {
    index <- match(ids, old)
    ids[!is.na(index)] <- names[index[!is.na(index)]]
    ids
  }
  dataset$main$records$assay_id <- replace_ids(dataset$main$records$assay_id)
  dataset$assays$records$assay_id <- names
  dataset$intensities <- lapply(dataset$intensities, function(file) {
    file$metadata$assay_id <- replace_ids(file$metadata$assay_id)
    file
  })
  dataset
}

rename_probes <- function(dataset, old, edited) {
  if (nrow(old) != nrow(edited) || !identical(old$channel_id, edited$channel_id)) {
    stop("Existing probe channels cannot be changed; add a new probe instead.")
  }
  check_names(edited$target_id, "Target IDs")
  if (any(grepl("[.;]", edited$target_id)) || anyDuplicated(probe_keys(edited))) {
    stop("Use unique target/channel pairs and target IDs without dots or semicolons.")
  }
  mapping <- unique(data.frame(old = old$target_id, new = edited$target_id))
  if (anyDuplicated(mapping$old) || anyDuplicated(mapping$new)) {
    stop("Rename every channel of a target consistently, without merging targets.")
  }
  replacements <- setNames(probe_keys(edited), probe_keys(old))
  dataset$assays$records$probe_combination <- vapply(
    strsplit(dataset$assays$records$probe_combination, ";", fixed = TRUE),
    function(tokens) paste(unname(replacements[tokens]), collapse = ";"), ""
  )
  index <- match(dataset$main$records$target_id, mapping$old)
  dataset$main$records$target_id[!is.na(index)] <- mapping$new[index[!is.na(index)]]
  dataset
}

assign_assay <- function(dataset, wells, assay_id, reset_results = FALSE) {
  assay <- dataset$assays$records[dataset$assays$records$assay_id == assay_id, , drop = FALSE]
  if (nrow(assay) != 1L) stop("Select one existing assay.")
  targets <- unique(split_probes(assay$probe_combination)$target_id)
  for (well in wells) {
    rows <- dataset$main$records[dataset$main$records$well_id == well, , drop = FALSE]
    samples <- unique(rows$sample_id)
    if (length(samples) != 1L) stop("Assay assignment requires exactly one sample per selected well.")
    if (!reset_results && all(rows$assay_id == assay_id)) next
    # A different assay changes the result's meaning: do not carry results across.
    replacement <- data.frame(well_id = well, sample_id = samples, assay_id = assay_id,
      target_id = targets, counts_positive = NA_character_, counts_negative = NA_character_,
      concentration_reaction_cp_ul = NA_real_, threshold_if_threshold = NA_real_)
    dataset$main$records <- rbind(dataset$main$records[dataset$main$records$well_id != well, ], replacement)
    dataset$intensities <- lapply(dataset$intensities, function(file) {
      if (file$metadata$well_id == well) file$metadata$assay_id <- assay_id
      file
    })
  }
  rownames(dataset$main$records) <- NULL
  dataset
}
