# Each browser session owns its loaded dataset, edits, and validated download.
ddes_server <- function(demo_path) {
  force(demo_path)
  function(input, output, session) {
    state <- shiny::reactiveValues(dataset = NULL, probes = NULL, download = NULL,
                                  message = "Load a dataset to begin.")
    attempt <- function(expr) {
      tryCatch(force(expr), error = function(e) {
        state$message <- conditionMessage(e)
        shiny::showNotification(conditionMessage(e), type = "error", duration = NULL)
      })
    }
    save <- function(dataset, message) {
      ddes::write_main(dataset$main)
      ddes::write_assays(dataset$assays)
      state$dataset <- dataset
      state$download <- NULL
      state$message <- message
    }
    load_dataset <- function(dataset) {
      probes <- split_probes(dataset$assays$records$probe_combination)
      check_names(dataset$assays$records$assay_id, "Assay IDs", unique = TRUE)
      save(dataset, paste("Loaded", nrow(dataset$main$records), "results and",
                          length(dataset$intensities), "intensity files."))
      state$probes <- probes
      shiny::updateTabsetPanel(session, "tabs", selected = "run")
    }
    table_input <- function(value, fallback) {
      if (is.null(value)) {
        fallback$selected <- rep(FALSE, nrow(fallback))
        return(fallback)
      }
      result <- rhandsontable::hot_to_r(value)
      if (nrow(result) != nrow(fallback)) stop("Use the Add/Delete buttons to change rows.")
      result
    }
    selected <- function(table) which(!is.na(table$selected) & table$selected)
    widget <- function(rows, readonly = character()) {
      rows$selected <- rep(FALSE, nrow(rows))
      labels <- c(target_id = "Target ID", channel_id = "Channel ID", assay_id = "Assay ID",
                  probe_combination = "Probes", probe_concentrations_nm = "Concentrations (nM)",
                  well_id = "Well", sample_id = "Sample ID", selected = "Select")
      table <- rhandsontable::rhandsontable(rows, rowHeaders = NULL, contextMenu = FALSE,
        stretchH = "all", colHeaders = unname(labels[names(rows)]), manualColumnResize = TRUE,
        autoColumnSize = FALSE)
      for (name in setdiff(names(rows), "selected")) {
        table <- rhandsontable::hot_col(table, match(name, names(rows)), type = "text",
          readOnly = name %in% readonly,
          width = switch(name, probe_combination = 200, probe_concentrations_nm = 150, 100))
      }
      rhandsontable::hot_col(table, ncol(rows), type = "checkbox", width = 60)
    }
    metadata_table <- function(metadata) {
      data.frame(field = names(metadata), value = unlist(metadata, use.names = FALSE))
    }

    shiny::observeEvent(input$start_demo, attempt({
      load_dataset(ddes::read_archive(demo_path))
    }))
    shiny::observeEvent(input$start, attempt({
      dataset <- switch(input$platform,
        DDES = {
          if (is.null(input$ddes_upload)) stop("Upload a DDES archive first.")
          ddes::read_archive(input$ddes_upload$datapath)
        },
        QIAcuity = {
          if (is.null(input$qi_results) || is.null(input$qi_layout) || is.null(input$qi_rfu)) {
            stop("Upload the analysis CSV, plate layout CSV, and RFU ZIP.")
          }
          convert_qiacuity(input$qi_results$datapath, input$qi_layout$datapath, input$qi_rfu$datapath)
        },
        "QX200 Quantasoft" = {
          if (is.null(input$qx_results) || is.null(input$qx_amplitudes)) stop("Upload results and amplitude CSVs.")
          convert_qx200(input$qx_results$datapath, input$qx_results$name, input$qx_amplitudes)
        }, stop("Select an input format."))
      load_dataset(dataset)
    }))

    output$status <- shiny::renderText(state$message)
    output$run_fields <- shiny::renderUI({
      shiny::req(state$dataset)
      m <- state$dataset$main$metadata
      shiny::tagList(
        shiny::textInput("experiment_id", "Experiment ID", m$experiment_id),
        shiny::textInput("run_id", "Run ID", m$run_id),
        shiny::textInput("instrument", "Instrument", m$instrument),
        shiny::textInput("software", "Instrument Software Version", m$instrument_software_version),
        shiny::textInput("plate", "Plate or Chip Type", m$plate_type),
        shiny::actionButton("save_run", "Save Run Information"))
    })
    shiny::observeEvent(input$save_run, attempt({
      shiny::req(state$dataset)
      x <- state$dataset
      m <- x$main$metadata
      m$experiment_id <- input$experiment_id
      m$run_id <- input$run_id
      m$instrument <- input$instrument
      m$instrument_software_version <- input$software
      m$plate_type <- input$plate
      x$main$metadata <- m
      x$assays$metadata[c("experiment_id", "run_id")] <- m[c("experiment_id", "run_id")]
      x$intensities <- lapply(x$intensities, function(file) {
        file$metadata[c("experiment_id", "run_id")] <- m[c("experiment_id", "run_id")]
        file
      })
      save(x, "Run information saved.")
    }))

    output$probes <- rhandsontable::renderRHandsontable({
      shiny::req(state$probes)
      widget(state$probes, "channel_id")
    })
    shiny::observeEvent(input$add_probe, attempt({
      shiny::req(state$dataset)
      p <- data.frame(target_id = input$probe_target, channel_id = input$probe_channel)
      check_names(unlist(p, use.names = FALSE), "Target and channel IDs")
      if (any(grepl("[.;]", unlist(p))) || probe_keys(p) %in% probe_keys(state$probes)) {
        stop("Use a new target/channel pair without dots or semicolons.")
      }
      state$probes <- rbind(state$probes, p)
      state$message <- "Probe added. Select it to create an assay; unused probes are not stored in DDES."
    }))
    shiny::observeEvent(input$delete_probe, attempt({
      shiny::req(state$dataset)
      p <- table_input(input$probes, state$probes)
      indexes <- selected(p)
      used <- unlist(strsplit(state$dataset$assays$records$probe_combination, ";", fixed = TRUE))
      if (any(probe_keys(state$probes[indexes, , drop = FALSE]) %in% used)) stop("An assay still uses a selected probe.")
      state$probes <- state$probes[setdiff(seq_len(nrow(state$probes)), indexes), , drop = FALSE]
      state$message <- "Selected unused probes deleted."
    }))
    shiny::observeEvent(input$save_probes, attempt({
      shiny::req(state$dataset)
      p <- table_input(input$probes, state$probes)[c("target_id", "channel_id")]
      x <- rename_probes(state$dataset, state$probes, p)
      save(x, "Probe names saved in assays and reaction summaries.")
      state$probes <- p
    }))

    output$assays <- rhandsontable::renderRHandsontable({
      shiny::req(state$dataset)
      widget(state$dataset$assays$records, "probe_combination")
    })
    shiny::observeEvent(input$save_assays, attempt({
      shiny::req(state$dataset)
      rows <- table_input(input$assays, state$dataset$assays$records)
      x <- rename_assays(state$dataset, rows$assay_id)
      x$assays$records$probe_concentrations_nm <- rows$probe_concentrations_nm
      save(x, "Assay names and concentrations saved.")
    }))
    shiny::observeEvent(input$add_assay, attempt({
      shiny::req(state$dataset)
      check_names(input$assay_name, "Assay ID")
      if (input$assay_name %in% state$dataset$assays$records$assay_id) stop("Assay ID already exists.")
      p <- table_input(input$probes, state$probes)
      indexes <- selected(p)
      if (!length(indexes)) stop("Select at least one saved probe.")
      x <- state$dataset
      x$assays$records <- rbind(x$assays$records, data.frame(assay_id = input$assay_name,
        probe_combination = paste(probe_keys(state$probes[indexes, , drop = FALSE]), collapse = ";"),
        probe_concentrations_nm = paste(rep("NA", length(indexes)), collapse = ";")))
      save(x, "Assay added. Enter concentrations and save when available.")
    }))
    shiny::observeEvent(input$replace_assay_probes, attempt({
      shiny::req(state$dataset)
      x <- state$dataset
      assay_index <- selected(table_input(input$assays, x$assays$records))
      probe_indexes <- selected(table_input(input$probes, state$probes))
      if (length(assay_index) != 1L || !length(probe_indexes)) stop("Select one assay and at least one saved probe.")
      combination <- paste(probe_keys(state$probes[probe_indexes, , drop = FALSE]), collapse = ";")
      if (identical(combination, x$assays$records$probe_combination[assay_index])) return(invisible(NULL))
      assay_id <- x$assays$records$assay_id[assay_index]
      x$assays$records$probe_combination[assay_index] <- combination
      x$assays$records$probe_concentrations_nm[assay_index] <- paste(rep("NA", length(probe_indexes)), collapse = ";")
      wells <- unique(x$main$records$well_id[x$main$records$assay_id == assay_id])
      if (any(x$main$records$well_id %in% wells & x$main$records$assay_id != assay_id)) {
        stop("Cannot replace probes for wells containing multiple assays.")
      }
      x <- assign_assay(x, wells, assay_id, reset_results = TRUE)
      save(x, "Assay probes replaced. Probe concentrations and affected reported results are now missing.")
    }))
    shiny::observeEvent(input$delete_assay, attempt({
      shiny::req(state$dataset)
      x <- state$dataset
      rows <- table_input(input$assays, x$assays$records)
      indexes <- selected(rows)
      used <- c(x$main$records$assay_id, vapply(x$intensities, function(file) file$metadata$assay_id, ""))
      if (any(x$assays$records$assay_id[indexes] %in% used)) stop("A reaction or intensity file still uses a selected assay.")
      x$assays$records <- x$assays$records[setdiff(seq_len(nrow(x$assays$records)), indexes), , drop = FALSE]
      save(x, "Selected unused assays deleted.")
    }))

    output$samples <- rhandsontable::renderRHandsontable({
      shiny::req(state$dataset)
      widget(sample_table(state$dataset), c("well_id", "assay_id"))
    })
    shiny::observe({
      shiny::req(state$dataset)
      shiny::updateSelectInput(session, "sample_assay", choices = state$dataset$assays$records$assay_id)
      wells <- vapply(state$dataset$intensities, function(file) file$metadata$well_id, "")
      current <- shiny::isolate(input$intensity_well)
      shiny::updateSelectInput(session, "intensity_well", choices = wells,
                              selected = if (length(current) == 1L && current %in% wells) current else head(wells, 1))
    })
    shiny::observeEvent(input$save_samples, attempt({
      shiny::req(state$dataset)
      x <- state$dataset
      old <- sample_table(x)
      rows <- table_input(input$samples, old)
      check_names(rows$sample_id, "Sample IDs")
      if (!identical(rows$well_id, old$well_id) || !identical(rows$assay_id, old$assay_id)) stop("Only sample IDs are editable here.")
      # Match the original three columns so wells with multiple result groups stay separate.
      records <- x$main$records
      for (i in seq_len(nrow(old))) {
        index <- records$well_id == old$well_id[i] & records$sample_id == old$sample_id[i] & records$assay_id == old$assay_id[i]
        x$main$records$sample_id[index] <- rows$sample_id[i]
      }
      save(x, "Sample IDs saved.")
    }))
    shiny::observeEvent(input$apply_assay, attempt({
      shiny::req(state$dataset)
      rows <- table_input(input$samples, sample_table(state$dataset))
      wells <- unique(rows$well_id[selected(rows)])
      if (!length(wells)) stop("Select at least one well.")
      x <- assign_assay(state$dataset, wells, input$sample_assay)
      save(x, "Assay assigned. Results for changed assays are now missing; fluorescence is unchanged.")
    }))

    output$main_metadata <- shiny::renderTable({ shiny::req(state$dataset); metadata_table(state$dataset$main$metadata) })
    output$main_records <- shiny::renderTable({ shiny::req(state$dataset); state$dataset$main$records }, digits = 8)
    output$assays_metadata <- shiny::renderTable({ shiny::req(state$dataset); metadata_table(state$dataset$assays$metadata) })
    output$assays_records <- shiny::renderTable({ shiny::req(state$dataset); state$dataset$assays$records })
    intensity <- shiny::reactive({
      shiny::req(state$dataset, input$intensity_well)
      wells <- vapply(state$dataset$intensities, function(file) file$metadata$well_id, "")
      index <- match(input$intensity_well, wells)
      shiny::req(!is.na(index))
      state$dataset$intensities[[index]]
    })
    output$intensity_metadata <- shiny::renderTable(metadata_table(intensity()$metadata))
    output$intensity_records <- shiny::renderTable(head(intensity()$records, 10), digits = 8)

    shiny::observeEvent(input$prepare_download, attempt({
      shiny::req(state$dataset)
      state$download <- ddes::archive_to_raw(state$dataset)
      state$message <- "All components validated. Download is ready."
    }))
    output$download_link <- shiny::renderUI({
      shiny::req(state$download)
      shiny::downloadButton("download", "Download DDES Archive")
    })
    output$download <- shiny::downloadHandler(
      filename = function() "dataset.ddes",
      contentType = "application/zip",
      content = function(path) {
        shiny::req(state$download)
        # Shiny owns this temporary destination; the bindings supply validated ZIP bytes.
        writeBin(state$download, path)
      }
    )
  }
}
