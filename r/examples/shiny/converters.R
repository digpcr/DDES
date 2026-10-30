# Vendor CSV mappings used by the DDES converter and annotation workflow.

vendor_csv <- function(file, rfu = FALSE) {
  lines <- readLines(file, warn = FALSE, encoding = "UTF-8")
  if (!length(lines)) stop("Empty vendor CSV.")
  lines[1] <- sub("^\ufeff", "", lines[1])
  separator <- ","
  if (grepl("^sep=[,;]$", lines[1], ignore.case = TRUE)) {
    separator <- substring(lines[1], 5)
    lines <- lines[-1]
  } else if (rfu && !grepl("Well", lines[1], fixed = TRUE)) {
    lines <- lines[-1] # RFU exports may have one descriptive preamble line.
  }
  connection <- textConnection(lines)
  on.exit(close(connection))
  widths <- utils::count.fields(connection, sep = separator, quote = "\"", comment.char = "")
  if (!length(widths) || any(widths != widths[1], na.rm = TRUE)) {
    stop("Vendor CSV rows must have the same number of fields as their header.")
  }
  utils::read.table(text = lines, header = TRUE, sep = separator, quote = "\"",
    comment.char = "", colClasses = "character", na.strings = NULL, check.names = TRUE,
    row.names = NULL, fill = FALSE, stringsAsFactors = FALSE)
}

require_columns <- function(rows, columns) {
  missing <- setdiff(columns, names(rows))
  if (length(missing)) stop(paste("Missing vendor columns:", paste(missing, collapse = ", ")))
  if (!nrow(rows)) stop("Vendor CSV has no data rows.")
}

missing_token <- function(values) is.na(values) | values == "" | values == "NA"

vendor_number <- function(values, field, optional = TRUE) {
  missing <- missing_token(values)
  result <- suppressWarnings(as.numeric(values))
  if (any(!missing & !is.finite(result)) || (!optional && any(missing))) {
    stop(paste(field, "must contain finite numbers", if (optional) "or empty/NA values." else "without missing values."))
  }
  result[missing] <- NA_real_
  result
}

vendor_count <- function(values) {
  values[missing_token(values)] <- NA_character_
  values
}

vendor_metadata <- function(run, instrument, plate) {
  list(ddes_version = "1.0.0", datetime = format(Sys.time(), "%Y%m%d%H%M"),
       experiment_id = "expID", run_id = run, ddes_type = "main",
       instrument = instrument, plate_type = plate, instrument_software_version = "NA")
}

vendor_dataset <- function(metadata, main, assays, intensities) {
  assay_metadata <- metadata[c("ddes_version", "datetime", "experiment_id", "run_id", "ddes_type")]
  assay_metadata$ddes_type <- "assays"
  dataset <- list(main = list(metadata = metadata, records = main),
    assays = list(metadata = assay_metadata, records = assays), intensities = intensities)
  # Validate all converted components before the UI replaces the loaded dataset.
  ddes::write_main(dataset$main)
  ddes::write_assays(dataset$assays)
  for (file in dataset$intensities) ddes::write_intensity(file)
  dataset
}

vendor_intensity <- function(metadata, well, assay, records) {
  metadata <- metadata[c("ddes_version", "datetime", "experiment_id", "run_id", "ddes_type")]
  metadata$ddes_type <- "intensity"
  metadata$well_id <- well
  metadata$assay_id <- assay
  list(metadata = metadata, records = records)
}

qi_channel <- function(channel) {
  colors <- c(C = "CRIMSON", G = "GREEN", Y = "YELLOW", R = "RED", O = "ORANGE")
  index <- match(toupper(channel), names(colors))
  channel[!is.na(index)] <- unname(colors[index[!is.na(index)]])
  full <- toupper(channel) %in% unname(colors)
  channel[full] <- toupper(channel[full])
  channel
}

convert_qiacuity <- function(results_path, layout_path, rfu_zip) {
  results <- vendor_csv(results_path)
  columns <- c("Well", "Sample.NTC.Control", "Reaction.Mix", "Target..Name.",
    "Partitions..Positive.", "Partitions..Negative.", "Conc...cp.µL...dPCR.reaction.", "Threshold")
  require_columns(results, c("Plate.ID", "Plate.type", columns))
  if (length(unique(results$Plate.ID)) != 1L || length(unique(results$Plate.type)) != 1L) {
    stop("Convert one plate and run at a time.")
  }
  metadata <- vendor_metadata(results$Plate.ID[1], "NA", results$Plate.type[1])
  main <- results[columns]
  names(main) <- c("well_id", "sample_id", "assay_id", "target_id", "counts_positive", "counts_negative",
                   "concentration_reaction_cp_ul", "threshold_if_threshold")
  main$counts_positive <- vendor_count(main$counts_positive)
  main$counts_negative <- vendor_count(main$counts_negative)
  main$concentration_reaction_cp_ul <- vendor_number(main$concentration_reaction_cp_ul, "Concentration")
  main$threshold_if_threshold <- vendor_number(main$threshold_if_threshold, "Threshold")

  layout <- vendor_csv(layout_path)
  require_columns(layout, c("Well", "Reaction.mix.name", paste0("Channel.", 1:5), paste0("Target.", 1:5)))
  layout <- unique(layout[!missing_token(layout$Reaction.mix.name), ])
  if (anyDuplicated(layout$Well)) stop("Conflicting plate-layout rows for one well.")
  wells <- unique(main$well_id)
  index <- match(wells, layout$Well)
  if (anyNA(index)) stop("Every results well needs a plate-layout assay.")
  combinations <- vapply(index, function(i) {
    targets <- unlist(layout[i, paste0("Target.", 1:5)], use.names = FALSE)
    channels <- unlist(layout[i, paste0("Channel.", 1:5)], use.names = FALSE)
    if (any(missing_token(targets) != missing_token(channels))) stop("Each layout probe needs both target and channel.")
    use <- !missing_token(targets)
    if (!any(use)) stop("Each assay needs at least one probe.")
    paste(unique(paste(targets[use], qi_channel(channels[use]), sep = ".")), collapse = ";")
  }, "")
  assay_ids <- layout$Reaction.mix.name[index]
  assays <- unique(data.frame(assay_id = assay_ids, probe_combination = combinations))
  if (anyDuplicated(assays$assay_id)) stop("One assay ID has conflicting probe definitions.")
  if (any(main$assay_id != assay_ids[match(main$well_id, wells)])) stop("Results and layout disagree on assay IDs.")
  assays$probe_concentrations_nm <- vapply(strsplit(assays$probe_combination, ";", fixed = TRUE),
    function(probes) paste(rep("NA", length(probes)), collapse = ";"), "")

  entries <- utils::unzip(rfu_zip, list = TRUE)$Name
  entries <- entries[grepl("RFU.*\\.csv$", basename(entries), ignore.case = TRUE)]
  if (!length(entries) || anyDuplicated(entries)) stop("RFU ZIP needs uniquely named RFU CSV files.")
  files <- lapply(entries, function(name) {
    connection <- unz(rfu_zip, name)
    on.exit(close(connection))
    rows <- vendor_csv(connection, rfu = TRUE)
    require_columns(rows, c("Well", "Partition", "Channel", "RFU"))
    rows[c("Well", "Partition", "Channel", "RFU")]
  })
  measurements <- do.call(rbind, files)
  if (!setequal(unique(measurements$Well), wells)) stop("RFU wells must match results wells.")
  intensities <- lapply(seq_along(wells), function(i) {
    rows <- measurements[measurements$Well == wells[i], ]
    records <- data.frame(partition_id = rows$Partition, channel_id = qi_channel(rows$Channel),
      fi_endpoint = vendor_number(rows$RFU, "RFU", optional = FALSE))
    if (anyDuplicated(records[c("partition_id", "channel_id")])) stop("Duplicate RFU partition/channel measurement.")
    vendor_intensity(metadata, wells[i], assay_ids[i], records)
  })
  vendor_dataset(metadata, main, assays, intensities)
}

convert_qx200 <- function(results_path, results_name, amplitude_files) {
  results <- vendor_csv(results_path)
  require_columns(results, c("Well", "Sample", "TargetType", "Target", "Positives", "Negatives", "Concentration", "Threshold"))
  channels <- c(Ch1Unknown = "Ch1", Ch2Unknown = "Ch2", Ch1 = "Ch1", Ch2 = "Ch2")
  if (any(!results$TargetType %in% names(channels))) stop("TargetType must identify Ch1 or Ch2 (including Ch1Unknown/Ch2Unknown).")
  wells <- unique(results$Well)
  combinations <- vapply(wells, function(well) {
    rows <- results[results$Well == well, ]
    paste(sort(unique(paste(rows$Target, unname(channels[rows$TargetType]), sep = "."))), collapse = ";")
  }, "", USE.NAMES = FALSE)
  assays <- data.frame(assay_id = unique(combinations), probe_combination = unique(combinations))
  assays$probe_concentrations_nm <- vapply(strsplit(assays$probe_combination, ";", fixed = TRUE),
    function(probes) paste(rep("NA", length(probes)), collapse = ";"), "")
  main <- data.frame(well_id = results$Well, sample_id = results$Sample,
    assay_id = combinations[match(results$Well, wells)], target_id = results$Target,
    counts_positive = vendor_count(results$Positives), counts_negative = vendor_count(results$Negatives),
    concentration_reaction_cp_ul = vendor_number(results$Concentration, "Concentration"),
    threshold_if_threshold = vendor_number(results$Threshold, "Threshold"))
  metadata <- vendor_metadata(tools::file_path_sans_ext(basename(results_name)), "QX200", "Droplets")
  used <- integer()
  intensities <- lapply(seq_along(wells), function(i) {
    index <- which(grepl(paste0("_", wells[i], "_"), amplitude_files$name, fixed = TRUE))
    if (length(index) != 1L) stop(paste("Expected exactly one amplitude filename containing", paste0("_", wells[i], "_")))
    used <<- c(used, index)
    rows <- vendor_csv(amplitude_files$datapath[index])
    if (ncol(rows) < 2L || !nrow(rows)) stop("Amplitude CSV needs two populated channel columns.")
    records <- data.frame(partition_id = rep(as.character(seq_len(nrow(rows))), 2),
      channel_id = rep(c("Ch1", "Ch2"), each = nrow(rows)),
      fi_endpoint = c(vendor_number(rows[[1]], "Channel 1 amplitude", FALSE),
                      vendor_number(rows[[2]], "Channel 2 amplitude", FALSE)))
    vendor_intensity(metadata, wells[i], combinations[i], records)
  })
  if (anyDuplicated(used) || !setequal(used, seq_len(nrow(amplitude_files)))) stop("Amplitude uploads must match results wells exactly.")
  vendor_dataset(metadata, main, assays, intensities)
}
