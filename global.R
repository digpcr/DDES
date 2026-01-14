################################################################################

#                      DDES Shiny Application: global                          #

################################################################################


## Created by: Wim Trypsteen
## Debugged by: Emile Roose
## v1.0 June 2025

################################################################################

## TO DO
# Clean up code
# filenaming DDES download gets (too) long to open + runID is changable -> no link with original plate (internal runID vs userdefined runID?)
# TO DO: check for allow for additional channels, now it is 5 channels
# partition coding 000001

## Set Global Variables
set.seed(1553)

# TO DO: Move to platform specific
instruments <<- c("QIAcuity One", "QIAcuity Four", "QIAcuity Eight", "QIAcuity Dx")
plate_types <<- c("Nanoplate 26K 8-well", "Nanoplate 26K 24-well", 
                  "Nanoplate 8.5K 24-well", "Nanoplate 8.5K 96-well")
#instrument_software_versions <- c("2.5.0.1","3.1","3.2") free text fields or dropdown?

experimentid <<- ""
runid <<- ""
plate_type <<- ""
instrument <<- ""
instrument_software_version <<- ""
ddes_version <<- "1.0"
special_chars <<- "^[a-zA-Z0-9_-]+$"
special_chars_2 <<- "^[a-zA-Z0-9_.-]+$"
special_chars_3 <<- "^[a-zA-Z0-9_. -]+$"
shared_processing <<- ""
demo_clicks <<- 0
upload_clicks <<- 0


## Packages
library(shiny)
library(rhandsontable)
library(stringr)
library(zip)
library(reshape2)
library(data.table)
library(shinyFeedback)
library(shinyalert)


## Functions used in server script

# Demo data retrieval
retrieve_demo_dataset_ddes <- function(demo_dataset_selected){
  
  # Initiate with progress bar
  withProgress(message = paste("Retrieving DDES Demo data set: ", demo_dataset_selected), value = 0,{
  progress_steps <- 3
    
  # Demo specific variables
  datetime <<- format(Sys.time(), "%Y%m%d%H%M")
  
  # Add the other channel ids in the software update from qiacuity
  # Add based on selector different demo data sets
 
  # Get filenames of ddes files
  ddes_folder <<- paste0("./demo_data/", demo_dataset_selected, "/")
  ddes_files <<- dir(ddes_folder,full.names = TRUE, include.dirs = FALSE)
  ddes_files_base <<- dir(ddes_folder)
 
  # Assign filenames to ddes format files
  ddes_main_file <<- ddes_files[grep(ddes_files,pattern = "*main*")]
  ddes_assay_file <<- ddes_files[grep(ddes_files,pattern = "*assay*")]
  ddes_intensity_files <<- ddes_files[grep(ddes_files,pattern = "*intensity*")]
  ddes_intensity_files_base <<- ddes_files_base[grep(ddes_files_base,pattern = "*intensity*")]
  
  # Read in ddes main file header and data
  ddes_import_main_head <- readLines(con = ddes_main_file, n = 10)
  ddes_import_main <- read.table(file = ddes_main_file,
                                 sep = "\t", stringsAsFactors = FALSE,
                                 comment.char = "#", header = TRUE)
  ddes_import_main$well_samplename <- paste(ddes_import_main$well,
                                            ddes_import_main$sample_ID, sep="_")
  
  # Construct main header data frame for viewing and editing
  ddes_import_main_head_df <- data.frame(do.call(rbind,str_split(ddes_import_main_head, "\t", n = 2)))
  colnames(ddes_import_main_head_df) <- c("header", "header_content")
  
  # Construct sample annotation table
  ddes_import_sample_annotation <- unique(ddes_import_main[,c("well_ID","assay_ID","sample_ID")])
  ddes_import_sample_annotation$selector <- "FALSE"
  rownames(ddes_import_sample_annotation) <- 1:dim(ddes_import_sample_annotation)[1]
  
  incProgress(1/progress_steps, detail = paste("Step", 1, "of", progress_steps))
  
  # Read in ddes assays file header and data
  ddes_import_assay_head <- readLines(con = ddes_assay_file, n = 8)
  ddes_import_assay <- read.table(file = ddes_assay_file , 
                                  comment.char = "#", sep="\t", header = TRUE)
 
  ddes_import_assay$target_number <- sapply(FUN = length, 
                                            strsplit(ddes_import_assay$probe_combination,
                                            split = ";"))
  ddes_import_assay$selector <- "FALSE"
  ddes_import_assay$selector <- as.logical(ddes_import_assay$selector)
  
  # Construct assay header data frame for viewing and editing
  ddes_import_assay_head_df <- data.frame(do.call(rbind,str_split(ddes_import_assay_head, "\t", n = 2)))
  colnames(ddes_import_assay_head_df) <- c("header", "header_content")
  
  # Construct probe info table
  ddes_import_probe <- t(as.data.frame(do.call(rbind, strsplit(ddes_import_assay$probe_combination, split= ";"))))
  ddes_import_probe <- as.data.frame(do.call(rbind, strsplit(ddes_import_probe, split= "\\.")))
  colnames(ddes_import_probe) <- c("target_ID","channel_ID")
  ddes_import_probe$selector <- "FALSE"
  ddes_import_probe$selector <- as.logical(ddes_import_probe$selector)
  
  incProgress(2/progress_steps, detail = paste("Step", 2, "of", progress_steps))
  
  # Read in ddes intensity files
  intensity_files_data_list <- list()
  intensity_files_header_list <- list()
  
  for(i in 1:length(ddes_intensity_files)){

    intensity_file_tmp <- ddes_intensity_files[i]

    intensity_files_header_list[[i]] <- readLines(con = intensity_file_tmp, n = 9)
    names(intensity_files_header_list)[i] <- gsub(ddes_intensity_files_base[i], pattern = ".tsv", replacement = "")
    
    intensity_files_data_list[[i]] <- read.table(file = intensity_file_tmp,
                                            sep = "\t", stringsAsFactors = FALSE,
                                            comment.char = "#", header = TRUE)
    intensity_files_data_list[[i]] <- intensity_files_data_list[[i]][!is.na(intensity_files_data_list[[i]]$FI_endpoint),]
    
    names(intensity_files_data_list)[i] <- gsub(ddes_intensity_files_base[i], pattern = ".tsv", replacement = "")
  }
  
  # Retrieve information in DDES header for objects to use in script
  runid <<- strsplit(ddes_import_main_head[5], split = "#run_ID\t")[[1]][2]
  experimentid <<- strsplit(ddes_import_main_head[4], split = "#experiment_ID\t")[[1]][2]
  instrument <<- strsplit(ddes_import_main_head[7], split = "#instrument\t")[[1]][2]
  plate_type <<- strsplit(ddes_import_main_head[8], split = "#plate_type\t")[[1]][2]
  instrument_software_version <<- strsplit(ddes_import_main_head[9], split = "#instrument_software_version\t")[[1]][2]
  well_ids <<- unique(ddes_import_main$well_ID)
  
  # Retrieve information in DDES info tables and make available in environment
  channel_id <<- ddes_import_probe$channel_ID
  assay_names <<- ddes_import_assay$assay_ID
  target_id <<- ddes_import_probe$target_ID
  targetchannel_id <<- unlist(strsplit(ddes_import_assay$probe_combination, split= ";"))

  # Make ddes format files available in environment
  ddes_import_main_head <<- ddes_import_main_head
  ddes_import_main_head_df <<- ddes_import_main_head_df
  ddes_import_main <<- ddes_import_main
  ddes_import_assay_head <<- ddes_import_assay_head
  ddes_import_assay_head_df <<- ddes_import_assay_head_df
  ddes_import_assay <<- ddes_import_assay
  ddes_import_probe <<- ddes_import_probe
  intensity_files_data_list <<- intensity_files_data_list
  intensity_files_header_list <<- intensity_files_header_list
  ddes_import_sample_annotation <<- ddes_import_sample_annotation
  
  incProgress(3/progress_steps, detail = paste("Step", 3, "of", progress_steps))
  
  # Close progress bar
  })
}


# Upload data retrieval
retrieve_upload_dataset_ddes <- function(){
  
  # Initiate with progress bar
  withProgress(message = paste("Using Uploaded Data Set: ", platform_selected), value = 0,{

  progress_steps <- 3
    
  # Demo specific variables
  datetime <<- format(Sys.time(), "%Y%m%d%H%M")
 
  # Get to the real filenames and not the temp file name
  to <<- file.path(dirname(ddes_zip_folder[['datapath']]), basename(ddes_zip_folder[['name']]))
  file.rename(ddes_zip_folder[['datapath']], to)
  ddes_zip_folder[['datapath']] <<- file.path(dirname(ddes_zip_folder[['datapath']]), basename(ddes_zip_folder[['name']]))
  
  # Create a temporary directory to unzip the files
  temp_dir <- tempdir()
  
  # Unzip the file
  unzip(ddes_zip_folder$datapath, exdir = temp_dir)
  
  # Get filenames of ddes files
  ddes_files <<- dir(temp_dir,full.names = TRUE, include.dirs = FALSE)
  ddes_files_base <<- dir(temp_dir)
  
  # Assign filenames to ddes format files
  ddes_main_file <<- ddes_files[grep(ddes_files,pattern = "*main*")]
  ddes_assay_file <<- ddes_files[grep(ddes_files,pattern = "*assay*")]
  ddes_intensity_files <<- ddes_files[grep(ddes_files,pattern = "*intensity*")]
  ddes_intensity_files_base <<- ddes_files_base[grep(ddes_files_base,pattern = "*intensity*")]
  
  # Read in ddes main file header and data
  ddes_import_main_head <- readLines(con = ddes_main_file, n = 10)
  ddes_import_main <- read.table(file = ddes_main_file,
                                 sep = "\t", stringsAsFactors = FALSE,
                                 comment.char = "#", header = TRUE)
  ddes_import_main$well_samplename <- paste(ddes_import_main$well,
                                            ddes_import_main$sample_ID, sep="_")
 
  # Construct main header data frame for viewing and editing
  ddes_import_main_head_df <- data.frame(do.call(rbind,str_split(ddes_import_main_head, "\t", n = 2)))
  colnames(ddes_import_main_head_df) <- c("header", "header_content")
  
  # Construct sample annotation table
  ddes_import_sample_annotation <- unique(ddes_import_main[,c("well_ID","assay_ID","sample_ID")])
  ddes_import_sample_annotation$selector <- "FALSE"
  rownames(ddes_import_sample_annotation) <- 1:dim(ddes_import_sample_annotation)[1]

  # Update progress bar
  incProgress(1/progress_steps, detail = paste("Step", 1, "of", progress_steps))
  
  # Read in ddes assays file header and data
  ddes_import_assay_head <- readLines(con = ddes_assay_file, n = 8)
  ddes_import_assay <- read.table(file = ddes_assay_file , 
                                  comment.char = "#", sep="\t", header = TRUE)
  
  ddes_import_assay$target_number <- sapply(FUN = length, 
                                            strsplit(ddes_import_assay$probe_combination,
                                                     split = ";"))
  ddes_import_assay$selector <- "FALSE"
  ddes_import_assay$selector <- as.logical(ddes_import_assay$selector)
  
  # Construct assay header data frame for viewing and editing
  ddes_import_assay_head_df <- data.frame(do.call(rbind,str_split(ddes_import_assay_head, "\t", n = 2)))
  colnames(ddes_import_assay_head_df) <- c("header", "header_content")
  
  # Construct probe info table
  ddes_import_probe <- t(as.data.frame(do.call(rbind, strsplit(ddes_import_assay$probe_combination, split= ";"))))
  ddes_import_probe <- as.data.frame(do.call(rbind, strsplit(ddes_import_probe, split= "\\.")))
  colnames(ddes_import_probe) <- c("target_ID","channel_ID")
  ddes_import_probe$selector <- "FALSE"
  ddes_import_probe$selector <- as.logical(ddes_import_probe$selector)

  incProgress(2/progress_steps, detail = paste("Step", 2, "of", progress_steps))
  
  # Read in ddes intensity files
  intensity_files_data_list <- list()
  intensity_files_header_list <- list()
  
  for(i in 1:length(ddes_intensity_files)){
    
    intensity_file_tmp <- ddes_intensity_files[i]
    
    intensity_files_header_list[[i]] <- readLines(con = intensity_file_tmp, n = 9)
    names(intensity_files_header_list)[i] <- gsub(ddes_intensity_files_base[i], pattern = ".tsv", replacement = "")
    
    intensity_files_data_list[[i]] <- read.table(file = intensity_file_tmp,
                                                 sep = "\t", stringsAsFactors = FALSE,
                                                 comment.char = "#", header = TRUE)
    intensity_files_data_list[[i]]<- intensity_files_data_list[[i]][!is.na(intensity_files_data_list[[i]]$FI_endpoint),]
    
    names(intensity_files_data_list)[i] <- gsub(ddes_intensity_files_base[i], pattern = ".tsv", replacement = "")
  }

  # Retrieve information in DDES header for objects to use in script
  runid <<- strsplit(ddes_import_main_head[5], split = "#run_ID\t")[[1]][2]
  experimentid <<- strsplit(ddes_import_main_head[4], split = "#experiment_ID\t")[[1]][2]
  instrument <<- strsplit(ddes_import_main_head[7], split = "#instrument\t")[[1]][2]
  plate_type <<- strsplit(ddes_import_main_head[8], split = "#plate_type\t")[[1]][2]
  instrument_software_version <<- strsplit(ddes_import_main_head[9], split = "#instrument_software_version\t")[[1]][2]
  well_ids <<- unique(ddes_import_main$well_ID)
  
  # Retrieve information in DDES info tables and make available in environment
  channel_id <<- ddes_import_probe$channel_ID
  assay_names <<- ddes_import_assay$assay_ID
  target_id <<- ddes_import_probe$target_ID
  targetchannel_id <<- unlist(strsplit(ddes_import_assay$probe_combination, split= ";"))
  
  # Make ddes format files available in environment
  ddes_import_main_head <<- ddes_import_main_head
  ddes_import_main_head_df <<- ddes_import_main_head_df
  ddes_import_main <<- ddes_import_main
  ddes_import_assay_head <<- ddes_import_assay_head
  ddes_import_assay_head_df <<- ddes_import_assay_head_df
  ddes_import_assay <<- ddes_import_assay
  ddes_import_probe <<- ddes_import_probe
  intensity_files_data_list <<- intensity_files_data_list
  intensity_files_header_list <<- intensity_files_header_list
  ddes_import_sample_annotation <<- ddes_import_sample_annotation
  
  # Update progress bar
  incProgress(3/progress_steps, detail = paste("Step", 3, "of", progress_steps))
  
  # Close progress bar
  })
  
}

retrieve_upload_dataset_qiacuity <- function(){
  
  ### Initiate with progress bar
  withProgress(message = paste("Using Uploaded Data Set: ", platform_selected), value = 0,{
    
  progress_steps <- 3
  
  # Update progress bar
  incProgress(1/progress_steps, detail = paste("Step", 1, "of", progress_steps))
  
  ### Session specific variables when started
  datetime <<- format(Sys.time(), "%Y%m%d%H%M")
  
  ### Get to the real filenames and not the temp file name
  to <<- file.path(dirname(qq_cur_res_file[['datapath']]), basename(qq_cur_res_file[['name']]))
  file.rename(qq_cur_res_file[['datapath']], to)
  qq_cur_res_file[['datapath']] <<- file.path(dirname(qq_cur_res_file[['datapath']]), basename(qq_cur_res_file[['name']]))
  
  to <<- file.path(dirname(qq_RFU_files[['datapath']]), basename(qq_RFU_files[['name']]))
  file.rename(qq_RFU_files[['datapath']], to)
  qq_RFU_files[['datapath']] <<- file.path(dirname(qq_RFU_files[['datapath']]), basename(qq_RFU_files[['name']]))
  
  to <<- file.path(dirname(qq_plate_layout_file[['datapath']]), basename(qq_plate_layout_file[['name']]))
  file.rename(qq_plate_layout_file[['datapath']], to)
  qq_plate_layout_file[['datapath']] <<- file.path(dirname(qq_plate_layout_file[['datapath']]), basename(qq_plate_layout_file[['name']]))
  

  # Create a temporary directory to unzip the files
  temp_dir_rfu <- tempdir()
  
  # Unzip the file
  unzip(qq_RFU_files$datapath, exdir = temp_dir_rfu)
  
  # Get filenames of ddes files
  qq_RFU_fnames <<- grep(dir(temp_dir_rfu, full.names = TRUE, include.dirs = FALSE), 
                         pattern = "RFU",value = TRUE)
  qq_RFU_base_fnames <<- grep(dir(temp_dir_rfu), pattern="RFU",value = TRUE)
  
  ### Get the datapaths
  qq_cur_res_fname <<- qq_cur_res_file$datapath
  qq_plate_layout_fname <<- qq_plate_layout_file$datapath
  #qq_RFU_fnames <<- qq_RFU_files$datapath
  #qq_RFU_base_fnames <<- qq_RFU_files$name
  
  print("Start DDES Conversion for QIAcuity data")
  
  # 0. Read in exported files from QIAcuity
  
  # Read in RFU files and combine in one
  export_file_tmp <- c()
  n_channels <<- length(qq_RFU_fnames)
  
  for(i in 1:length(qq_RFU_fnames)){

    export_file_tmp1 <- read.csv(file = qq_RFU_fnames[i], header = TRUE, 
                                 stringsAsFactors = FALSE, skip = 1)
    
    export_file_tmp <- rbind(export_file_tmp, export_file_tmp1)
  }
  export_file_tmp <<- export_file_tmp
  rm(export_file_tmp1)
  
  
  # Read in first line plate layout
  first_line <- readLines(qq_plate_layout_fname, n = 1)
  
  # Check if the first line contains sep=","
  if (grepl("sep", first_line)) {
    
  export_platelayout_tmp <<- read.csv(file = qq_plate_layout_fname, header = TRUE, skip = 1)
  
  } else {
  
  export_platelayout_tmp <<- read.csv(file = qq_plate_layout_fname, header = TRUE)
    
  }
  
  # Read in export current result file with all samples and all available channels selected
  first_line <- readLines(qq_cur_res_fname, n = 1)
  if (grepl("sep", first_line)) {
  export_current_results <<- read.csv(file=qq_cur_res_fname,
                                      header = TRUE, skip=1,
                                      stringsAsFactors = FALSE)
  } else {
    
    export_current_results <<- read.csv(file=qq_cur_res_fname,
                                        header = TRUE,
                                        stringsAsFactors = FALSE)
  }
  plate_info <<- export_current_results[1,c(1:3)]
  export_current_results <<- export_current_results[,-c(1:3)]
  

  ## 1. Main file
  print("Start DDES Conversion for QIAcuity data: Main File")
  
  # Retrieve additional information for header
  datetime <<- format(Sys.time(), "%Y%m%d%H%M")
  experimentid <<- "expID" 
  runid <<- plate_info$Plate.ID
  instrument <<- "NA"
  plate_type <<- plate_info$Plate.type
  instrument_software_version <<- "NA"

 
  ddes_main_head <- c("#header",
                      paste("#DDES_version", ddes_version, sep = "\t"),
                      paste("#datetime", datetime, sep = "\t"),
                      paste("#experiment_ID", experimentid, sep = "\t"),
                      paste("#run_ID", runid, sep = "\t"),
                      paste("#DDES_type", "main", sep = "\t"),
                      paste("#instrument", instrument, sep = "\t"),
                      paste("#plate_type", plate_type, sep = "\t"),
                      paste("#instrument_software_version", instrument_software_version, sep = "\t"),
                      "#data")
  
  # Retrieve info for main data columns (Qiagen gives valid filtered result table total partitions will not match lines in RFU file)
  ddes_main_colnames <- c("well_ID", "sample_ID", "assay_ID","target_ID",
                          "counts_positive", "counts_negative", "concentration_reaction_cp_µL","threshold_if_threshold")
  
  ddes_main_data <- export_current_results[,c("Well","Sample.NTC.Control",
                                              "Reaction.Mix", "Target..Name.",
                                              "Partitions..Positive.",
                                              "Partitions..Negative.","Conc...cp.µL...dPCR.reaction.","Threshold")]
  
  colnames(ddes_main_data) <- ddes_main_colnames
  ddes_main_data <- replace(ddes_main_data,ddes_main_data=="NA", "-")
  ddes_main_data <<- ddes_main_data
  
  # Construct sample annotation table
  ddes_import_sample_annotation <- unique(ddes_main_data[,c("well_ID","assay_ID","sample_ID")])
  ddes_import_sample_annotation$selector <- "FALSE"
  rownames(ddes_import_sample_annotation) <- 1:dim(ddes_import_sample_annotation)[1]
  well_ids <<- ddes_import_sample_annotation$well_ID
  
  ## 2. Assays file
  
  print("Start DDES Conversion for QIAcuity data: Assay File")
  
  # Initiate probes info object
  ddes_probe_info_colnames <- c("target_ID","channel_ID","selector")
  ddes_probe_info <- c()
  
  # Retrieve assay information: EMPTY REACTION MIX NAMES NOT ALLOWED (FILTERED OUT)
  assay_ids <<- unique(export_platelayout_tmp$Reaction.mix.name)
  assay_ids <<- assay_ids[ assay_ids != ""]
  n_assays <<- length(assay_ids)
  
  assays_detected <<- unique(export_platelayout_tmp[
    c("Reaction.mix.name",
      "Channel.1", "Target.1",
      "Channel.2", "Target.2",
      "Channel.3", "Target.3",
      "Channel.4", "Target.4",
      "Channel.5", "Target.5")])
  #"Channel.6", "Target.6"),
  #"Channel.7", "Target.7"),
  #"Channel.8", "Target.8")])
  
  assays_detected <<- assays_detected[which(assays_detected$Reaction.mix.name %in% assay_ids),]
  
  # Retrieve which probes are detected = target information
  probes_detected <<- unique(c(paste(assays_detected$Target.1,assays_detected$Channel.1,sep="_"),
                               paste(assays_detected$Target.2,assays_detected$Channel.2,sep="_"),
                               paste(assays_detected$Target.3,assays_detected$Channel.3,sep="_"),
                               paste(assays_detected$Target.4,assays_detected$Channel.4,sep="_"),
                               paste(assays_detected$Target.5,assays_detected$Channel.5,sep="_")))
  #paste(assays_detected$Target.6,assays_detected$Channel.6,sep="_"),
  #paste(assays_detected$Target.7,assays_detected$Channel.7,sep="_"),
  #paste(assays_detected$Target.8,assays_detected$Channel.8,sep="_")))
  if(any(grepl(pattern="NA_NA", x=probes_detected))==TRUE){
    
    probes_detected <- probes_detected[-grep(pattern="NA_NA", x=probes_detected)]
    
  } else if(any(grepl(pattern="NA.NA", x=probes_detected))==TRUE){
    
    probes_detected <- probes_detected[-grep(pattern="NA.NA", x=probes_detected)]
    
  } else {
    
    probes_detected <- probes_detected
  }
  
  # Construct probes info object
  for(a in 1:length(probes_detected)){

    probes_tmp <- strsplit(x = probes_detected[a], split = "_")
    channelid_tmp <- probes_tmp[[1]][2]
    target_name_tmp <- probes_tmp[[1]][1]
    
    probe_info_tmp <- c(target_name_tmp, channelid_tmp,"FALSE")
    ddes_probe_info <- rbind(ddes_probe_info,probe_info_tmp)
  }
  
  colnames(ddes_probe_info) <- ddes_probe_info_colnames
  rownames(ddes_probe_info) <- 1:length(probes_detected)
  ddes_probe_info <- as.data.frame(ddes_probe_info)
  probes_detected <<- probes_detected
  # Retrieve assay information
  ddes_assay_info_colnames <- c("assay_ID", "probe_combination","probe_concentrations_nM")
  ddes_assay_info <- c()
  
  for(b in 1:length(assay_ids)){

    assays_tmp <- assays_detected[assays_detected$Reaction.mix==assays_detected[b],]
    assay_name <- assays_tmp$Reaction.mix.name
    target_names <- unique(c(paste(assays_tmp$Target.1,assays_tmp$Channel.1,sep="."),
                             paste(assays_tmp$Target.2,assays_tmp$Channel.2,sep="."),
                             paste(assays_tmp$Target.3,assays_tmp$Channel.3,sep="."),
                             paste(assays_tmp$Target.4,assays_tmp$Channel.4,sep="."),
                             paste(assays_tmp$Target.5,assays_tmp$Channel.5,sep=".")))
    
    if(any(grepl(pattern="NA_NA", x=target_names))==TRUE){
      
      target_names <- target_names[-grep(pattern="NA_NA", x=target_names)]
      
    } else if(any(grepl(pattern="NA.NA", x=target_names))==TRUE){
      
      target_names <- target_names[-grep(pattern="NA.NA", x=target_names)]
      
    }    else {
      
      target_names <- target_names
    }

    probe_concentrations_nM <- rep(x="NA",length(target_names))
    
    assay_info_tmp <- c(assay_name, paste(target_names, collapse = ";"), 
                        paste(probe_concentrations_nM, collapse = ";"))
    
    ddes_assay_info <- rbind(ddes_assay_info, assay_info_tmp)
    
  }
  ddes_assay_info <- as.data.frame(ddes_assay_info)
  colnames(ddes_assay_info) <- ddes_assay_info_colnames
  rownames(ddes_assay_info) <- 1:length(assay_ids)
  target_names <<- target_names
  print(ddes_assay_info)
  ddes_import_assay <- ddes_assay_info
  print(ddes_import_assay)
  ddes_assay_details_header <- c("#header",
                                 paste("#DDES_version", ddes_version, sep = "\t"),
                                 paste("#datetime", datetime, sep = "\t"),
                                 paste("#experiment_ID", experimentid, sep = "\t"),
                                 paste("#run_ID", runid, sep = "\t"),
                                 paste("#DDES_type", "assays", sep = "\t"),
                                 "#data","#assay_information")
  
  # Create objects used in the server
  ddes_import_main_head <- ddes_main_head
  ddes_import_main <- ddes_main_data
  ddes_import_main$well_samplename <- paste(ddes_import_main$well,ddes_import_main$sample_ID,
                                            sep="_")
  ddes_import_assay_head <- ddes_assay_details_header
  ddes_import_assay$target_number <- sapply(FUN = length, 
                                            strsplit(ddes_import_assay$probe_combination,
                                                     split = ";"))
  ddes_import_assay$selector <- "FALSE"
  ddes_import_assay$selector <- as.logical(ddes_import_assay$selector)
  
  ddes_import_assay_head_df <- data.frame(do.call(rbind,str_split(ddes_import_assay_head, "\t", n = 2)))
  colnames(ddes_import_assay_head_df) <- c("header", "header_content")
  
  ddes_import_probe <- ddes_probe_info
  ddes_import_main_head_df <- data.frame(do.call(rbind,str_split(ddes_import_main_head, "\t", n = 2)))
  colnames(ddes_import_main_head_df) <- c("header", "header_content")
  
  # Update progress bar
  incProgress(2/progress_steps, detail = paste("Step", 2, "of", progress_steps))
  

  ## 3. INTENSITY FILES
  print("Start DDES Conversion for QIAcuity data: Intensity Files")
  
  # Initiate intensity file objects
  intensity_files_data_list <- list()
  intensity_files_header_list <- list()
  intensity_filenames <- paste0("DDES_", experimentid, "_", runid,"_", datetime, 
                                "_intensity_", well_ids,".tsv")
  
  # Initiate intensity column names and header names
  ddes_intensity_colnames <- c("partition_ID","channel_ID","FI_endpoint")
  ddes_header <- c("#header", "DDES_version","datetime", "experiment_ID", "run_ID", "DDES_type", "well_ID","assay_ID",
                   "#data")
  
  # Construct intensity files in DDES format
  for(i in 1:length(well_ids)){

    print(i)
    
    # Retrieve assay name
    assay_name_tmp <- export_platelayout_tmp[export_platelayout_tmp$Well==well_ids[i],]$Reaction.mix.name
    
    # Subset RFU data per well and convert to DDES format
    ddes_intensity_tmp <- export_file_tmp[export_file_tmp$Well==well_ids[i],]
    ddes_intensity_tmp_converted <- ddes_intensity_tmp[,c("Partition","Channel","RFU")]
    colnames(ddes_intensity_tmp_converted) <- ddes_intensity_colnames
    ddes_intensity_tmp_converted <- ddes_intensity_tmp_converted[!is.na(ddes_intensity_tmp_converted$RFU),]
    
    # Match channelID naming (R,Y,G,C,O in stead of Red,yellow,etc)

    ddes_intensity_tmp_converted$channel_ID <- gsub(ddes_intensity_tmp_converted$channel_ID,
                                                    pattern="^C$", replacement="CRIMSON")
    ddes_intensity_tmp_converted$channel_ID <- gsub(ddes_intensity_tmp_converted$channel_ID,
                                                    pattern="^G$", replacement="GREEN")
    ddes_intensity_tmp_converted$channel_ID <- gsub(ddes_intensity_tmp_converted$channel_ID,
                                                    pattern="^Y$", replacement="YELLOW")
    ddes_intensity_tmp_converted$channel_ID <- gsub(ddes_intensity_tmp_converted$channel_ID,
                                                    pattern="^R$", replacement="RED")
    ddes_intensity_tmp_converted$channel_ID <- gsub(ddes_intensity_tmp_converted$channel_ID,
                                                    pattern="^O$", replacement="ORANGE")
    
    # Construct header lines for each well (could also use ">")
    ddes_header_lines_well <- c("#head",
                                paste("#DDES_version", ddes_version, sep = "\t"),
                                paste("#datetime", datetime, sep = "\t"),
                                paste("#experiment_ID", experimentid, sep = "\t"),
                                paste("#run_ID", runid, sep = "\t"),
                                paste("#DDES_type", "intensity", sep = "\t"),
                                paste("#well_ID", well_ids[i], sep = "\t"),
                                paste("#assay_ID", assay_name_tmp, sep = "\t"),
                                "#data")
    
    # Add the data to the intensity file objects
    intensity_files_data_list[[i]] <- ddes_intensity_tmp_converted
    names(intensity_files_data_list)[i] <- gsub(intensity_filenames[i], pattern = ".tsv", replacement = "")
    intensity_files_header_list[[i]] <- ddes_header_lines_well
    names(intensity_files_header_list)[i] <- gsub(intensity_filenames[i], pattern = ".tsv", replacement = "")
  
  }
 
  
  ## 4. Make all necessary information available for further processing
  # Retrieve information in DDES header for objects to use in script
  runid <<- strsplit(ddes_import_main_head[5], split = "#run_ID\t")[[1]][2]
  experimentid <<- strsplit(ddes_import_main_head[4], split = "#experiment_ID\t")[[1]][2]
  instrument <<- strsplit(ddes_import_main_head[7], split = "#instrument\t")[[1]][2]
  plate_type <<- strsplit(ddes_import_main_head[8], split = "#plate_type\t")[[1]][2]
  instrument_software_version <<- strsplit(ddes_import_main_head[9], split = "#instrument_software_version\t")[[1]][2]
  well_ids <<- unique(ddes_import_main$well_ID)
  
  # Retrieve information in DDES info tables and make available in environment
  channel_id <<- ddes_import_probe$channel_ID
  assay_names <<- ddes_import_assay$assay_ID
  target_id <<- ddes_import_probe$target_ID
  targetchannel_id <<- unlist(strsplit(ddes_import_assay$probe_combination, split= ";"))
  
  # Make ddes format files available in environment
  ddes_import_main_head <<- ddes_import_main_head
  ddes_import_main_head_df <<- ddes_import_main_head_df
  ddes_import_main <<- ddes_import_main
  ddes_import_assay_head <<- ddes_import_assay_head
  ddes_import_assay_head_df <<- ddes_import_assay_head_df
  ddes_import_assay <<- ddes_import_assay
  ddes_import_probe <<- ddes_import_probe
  intensity_files_data_list <<- intensity_files_data_list
  intensity_files_header_list <<- intensity_files_header_list
  ddes_import_sample_annotation <<- ddes_import_sample_annotation
  
  # Update progress bar
  incProgress(3/progress_steps, detail = paste("Step", 3, "of", progress_steps))
  
  # Close progress bar
  })
}

retrieve_upload_dataset_qx200_quantasoft <- function(){
  
  ### Initiate with progress bar
  withProgress(message = paste("Using Uploaded Data Set: ", platform_selected), value = 0,{
    
    progress_steps <- 3
    
    # Update progress bar
    incProgress(1/progress_steps, detail = paste("Step", 1, "of", progress_steps))
    
    # Session specific variables when started
    datetime <<- format(Sys.time(), "%Y%m%d%H%M")
    
    instruments <<- c("QX200")
    plate_types <<- c("Droplets")
    
    # Get to the real filenames and not the temp file name
    to <<- file.path(dirname(qx200_head_file[['datapath']]), basename(qx200_head_file[['name']]))
    file.rename(qx200_head_file[['datapath']], to)
    qx200_head_file[['datapath']] <<- file.path(dirname(qx200_head_file[['datapath']]), basename(qx200_head_file[['name']]))
    
    to <<- file.path(dirname(qx200_amplitude_files[['datapath']]), basename(qx200_amplitude_files[['name']]))
    file.rename(qx200_amplitude_files[['datapath']], to)
    qx200_amplitude_files[['datapath']] <<- file.path(dirname(qx200_amplitude_files[['datapath']]), basename(qx200_amplitude_files[['name']]))
    
    #if (is.null(qq_cur_res_file)){return(NULL)}
    #if (is.null(qq_RFU_files)){return(NULL)}
    #if (is.null(qq_plate_layout_file)){return(NULL)}
    
    # Get the datapaths
    qx200_head_fname <<- qx200_head_file$datapath
    qx200_head_base_fname <<- gsub(qx200_head_file$name,replacement = "",pattern=".csv")
    
    qx200_amplitude_fnames <<- qx200_amplitude_files$datapath
    qx200_amplitude_base_fnames <<- qx200_amplitude_files$name
    
    print("Start DDES Conversion for QX200 data - Quantasoft")
    
    
    # 0. Read in head file from QX200 - Quantasoft
    
    # Read in qx200_head with check for first line sep
    first_line <- readLines(qx200_head_fname, n = 1)
    
    # Check if the first line contains sep=","
    if (grepl("sep", first_line)) {
      
      export_qx200_head <<- read.csv(file = qx200_head_fname, header = TRUE, skip = 1,  row.names=NULL)
      colnames(export_qx200_head) <- c(colnames(export_qx200_head)[2:dim(export_qx200_head)[2]], "NA")
      
    } else {
      
      export_qx200_head <<- read.csv(file = qx200_head_fname, header = TRUE, row.names=NULL)
      colnames(export_qx200_head) <- c(colnames(export_qx200_head)[2:dim(export_qx200_head)[2]], "NA")
      
    }
    

    # 1. Main file
  
    print("Start DDES Conversion for QX200 - Quantasoft data: Main File")
  
    # Retrieve additional information for header
    datetime <<- format(Sys.time(), "%Y%m%d%H%M")
    experimentid <<- "expID"
    runid <<- qx200_head_base_fname
    instrument <<- "QX200"
    plate_type <<- "Droplets"
    instrument_software_version <<- "NA"
  
    ddes_main_head <- c("#header",
                        paste("#DDES_version", ddes_version, sep = "\t"),
                        paste("#datetime", datetime, sep = "\t"),
                        paste("#experiment_ID", experimentid, sep = "\t"),
                        paste("#run_ID", runid, sep = "\t"),
                        paste("#DDES_type", "main", sep = "\t"),
                        paste("#instrument", instrument, sep = "\t"),
                        paste("#plate_type", plate_type, sep = "\t"),
                        paste("#instrument_software_version", instrument_software_version, sep = "\t"),
                        "#data")
    
    # Retrieve info for main data columns
    ddes_main_colnames <- c("well_ID", "sample_ID", "assay_ID","target_ID",
                            "counts_positive", "counts_negative", "concentration_reaction_cp_µL","threshold_if_threshold")
    
    ddes_main_data <- export_qx200_head[,c("Well","Sample",
                                                "TargetType", "Target",
                                                "Positives",
                                                "Negatives","Concentration","Threshold")]
    
    colnames(ddes_main_data) <- ddes_main_colnames
    ddes_main_data$assay_ID <- "-"
    ddes_main_data <<- ddes_main_data
    
    # To Do Set No Call in concentration column to zero?
    

    # 2. Assay file
    
    print("Start DDES Conversion for QX200 - Quantasoft data: Assay File")
    
    # Retrieve assay information
    assay_info <<- unique(export_qx200_head[,c("Well","TargetType","Target")])
    assay_info$Target_TargetType <<- paste(assay_info$Target,assay_info$TargetType, sep="__")
    
    # Initiate probes info object
    ddes_probe_info_colnames <- c("target_ID","channel_ID","selector")
    ddes_probe_info <- c()
    
    # Retrieve which probes are detected
    probes_detected <<- unique(assay_info$Target_TargetType)
    
    # Construct probes info object
    for(a in 1:length(probes_detected)){
      
      probes_tmp <- strsplit(x = probes_detected[a], split = "__")
      channelid_tmp <- probes_tmp[[1]][2]
      target_name_tmp <- probes_tmp[[1]][1]
      
      probe_info_tmp <- c(target_name_tmp, channelid_tmp,"FALSE")
      ddes_probe_info <- rbind(ddes_probe_info,probe_info_tmp)
    }
    
    colnames(ddes_probe_info) <- ddes_probe_info_colnames
    rownames(ddes_probe_info) <- 1:length(probes_detected)
    ddes_probe_info <<- as.data.frame(ddes_probe_info)

    # Initiate assay info object
    ddes_assay_info_colnames <- c("assay_ID", "probe_combination","probe_concentrations_nM")
    ddes_assay_info <- c()
    
    # Retrieve which assays are detected (make assay IDs)
    #assay_ids <- reshape2::melt(assay_info[,c("Well","Target_TargetType")], id.vars = "Well")
    assay_ids_melt <- reshape2::dcast(assay_info, Well ~ "melt", fun.aggregate = function(x) paste(x, collapse = ";"))
    assay_ids_melt$melt <- gsub(assay_ids_melt$melt, pattern = "__", replacement = ".")
    assay_ids <- unique(assay_ids_melt$melt)
    assay_ids <<- assay_ids[ assay_ids != ""]
    n_assays <<- length(assay_ids)
    assay_ids_melt$name <- gsub(assay_ids_melt$melt, pattern = "Ch1Unknown|Ch2Unknown|\\.", replacement = "")
   
    # Construct assay info object
    for(b in 1:length(assay_ids)){
      
      assay_tmp <- assay_ids[b]
      assay_name <- gsub(assay_tmp, pattern = "Ch1Unknown|Ch2Unknown|\\.", replacement = "")
      targets_tmp <- strsplit(assay_tmp, split=";")
      target_names <- targets_tmp[[1]]
      
      probe_concentrations_nM <- rep(x="NA",length(target_names))
      
      assay_info_tmp <- c(assay_name, paste(target_names, collapse = ";"), 
                          paste(probe_concentrations_nM, collapse = ";"))
      
      ddes_assay_info <- rbind(ddes_assay_info, assay_info_tmp)
      
    }
    
    ddes_assay_info <- as.data.frame(ddes_assay_info)
    colnames(ddes_assay_info) <- ddes_assay_info_colnames
    rownames(ddes_assay_info) <- 1:length(assay_ids)
    
    target_names <<- target_names
    
    ddes_import_assay <<- ddes_assay_info
    
    print(ddes_import_assay)
    
    ddes_assay_details_header <- c("#header",
                                   paste("#DDES_version", ddes_version, sep = "\t"),
                                   paste("#datetime", datetime, sep = "\t"),
                                   paste("#experiment_ID", experimentid, sep = "\t"),
                                   paste("#run_ID", runid, sep = "\t"),
                                   paste("#DDES_type", "assays", sep = "\t"),
                                   "#data","#assay_information")
    # update assay naming in main
    idx <- match(ddes_main_data$well_ID, assay_ids_melt$Well)
    ddes_main_data$assay_ID[ddes_main_data$assay_ID == "-" & !is.na(idx)] <- assay_ids_melt$name[idx][ddes_main_data$assay_ID == "-" & !is.na(idx)]
 
    # Construct sample annotation table
    ddes_import_sample_annotation <- unique(ddes_main_data[,c("well_ID","assay_ID","sample_ID")])
    ddes_import_sample_annotation$selector <- "FALSE"
    rownames(ddes_import_sample_annotation) <- 1:dim(ddes_import_sample_annotation)[1]
    well_ids <<- ddes_import_sample_annotation$well_ID
    
    
    # Create objects used in the server
    ddes_import_main_head <- ddes_main_head
    ddes_import_main <- ddes_main_data
    ddes_import_main$well_samplename <- paste(ddes_import_main$well,ddes_import_main$sample_ID,
                                              sep="_")
    ddes_import_assay_head <- ddes_assay_details_header
    ddes_import_assay$target_number <- sapply(FUN = length, 
                                              strsplit(ddes_import_assay$probe_combination,
                                                       split = ";"))
    ddes_import_assay$selector <- "FALSE"
    ddes_import_assay$selector <- as.logical(ddes_import_assay$selector)
    
    ddes_import_assay_head_df <- data.frame(do.call(rbind,str_split(ddes_import_assay_head, "\t", n = 2)))
    colnames(ddes_import_assay_head_df) <- c("header", "header_content")
    
    ddes_import_probe <- as.data.frame(ddes_probe_info)
    ddes_import_main_head_df <- data.frame(do.call(rbind,str_split(ddes_import_main_head, "\t", n = 2)))
    colnames(ddes_import_main_head_df) <- c("header", "header_content")
    
    # Update progress bar
    incProgress(2/progress_steps, detail = paste("Step", 2, "of", progress_steps))
    
    
    # 3. INTENSITY FILES
    
    print("Start DDES Conversion for QX200 - Quantasoft: Intensity Files")
    
    # Initiate intensity file objects
    intensity_files_data_list <- list()
    intensity_files_header_list <- list()
    intensity_filenames <- paste0("DDES_", experimentid, "_", runid,"_", datetime, 
                                  "_intensity_", well_ids,".tsv")
    
    # Initiate intensity column names and header names
    ddes_intensity_colnames <- c("partition_ID","channel_ID","FI_endpoint")
    ddes_header <- c("#header", "DDES_version","datetime", "experiment_ID", "run_ID", "DDES_type", "well_ID","assay_ID",
                     "#data")
    
    # Construct intensity files in DDES format
    for(i in 1:length(well_ids)){
      
      print(i)
      
      # Retrieve assay name
      assay_name_tmp <- assay_ids_melt[assay_ids_melt$Well==well_ids[i],]$melt
      
      # Subset RFU data per well and convert to DDES format
      ddes_intensity_tmp_qx200 <- grep(qx200_amplitude_fnames, pattern = paste0("_",well_ids[i],"_"),value = TRUE)
      ddes_intensity_tmp_qx200 <- read.csv(ddes_intensity_tmp_qx200, header = TRUE, stringsAsFactors = FALSE)
      
      #ddes_intensity_tmp_qx200$Channel1 <- "Ch1"
      #ddes_intensity_tmp_qx200$Channel2 <- "Ch2"
      
      ddes_intensity_tmp_converted <- data.frame(
        Channel = c(rep("Ch1",nrow(ddes_intensity_tmp_qx200)),rep("Ch2",nrow(ddes_intensity_tmp_qx200))),
        RFU     = c(ddes_intensity_tmp_qx200[,1], ddes_intensity_tmp_qx200[,2]),
        Partition = 1:(nrow(ddes_intensity_tmp_qx200) * 2)
      )

      ddes_intensity_tmp_converted <- ddes_intensity_tmp_converted[,c(3,1,2)]
      colnames(ddes_intensity_tmp_converted) <- ddes_intensity_colnames
      

      # Construct header lines for each well (could also use ">")
      ddes_header_lines_well <- c("#head",
                                  paste("#DDES_version", ddes_version, sep = "\t"),
                                  paste("#datetime", datetime, sep = "\t"),
                                  paste("#experiment_ID", experimentid, sep = "\t"),
                                  paste("#run_ID", runid, sep = "\t"),
                                  paste("#DDES_type", "intensity", sep = "\t"),
                                  paste("#well_ID", well_ids[i], sep = "\t"),
                                  paste("#assay_ID", assay_name_tmp, sep = "\t"),
                                  "#data")
      
      # Add the data to the intensity file objects
      intensity_files_data_list[[i]] <- ddes_intensity_tmp_converted
      names(intensity_files_data_list)[i] <- gsub(intensity_filenames[i], pattern = ".tsv", replacement = "")
      intensity_files_header_list[[i]] <- ddes_header_lines_well
      names(intensity_files_header_list)[i] <- gsub(intensity_filenames[i], pattern = ".tsv", replacement = "")
      
    }

    
    ## 4. Make all necessary information available for further processing
 

    
    # Make ddes format files available in environment
    ddes_import_main_head <<- ddes_main_head
    ddes_import_main_head_df <<- ddes_import_main_head_df
    ddes_import_main <<- ddes_import_main
    ddes_import_assay_head <<- ddes_import_assay_head
    ddes_import_assay_head_df <<- ddes_import_assay_head_df
    ddes_import_assay <<- ddes_import_assay
    ddes_import_probe <<- ddes_import_probe
    intensity_files_data_list <<- intensity_files_data_list
    intensity_files_header_list <<- intensity_files_header_list
    ddes_import_sample_annotation <<- ddes_import_sample_annotation
    
    # Retrieve information in DDES header for objects to use in script
    runid <<- strsplit(ddes_import_main_head[5], split = "#run_ID\t")[[1]][2]
    experimentid <<- strsplit(ddes_import_main_head[4], split = "#experiment_ID\t")[[1]][2]
    instrument <<- strsplit(ddes_import_main_head[7], split = "#instrument\t")[[1]][2]
    plate_type <<- strsplit(ddes_import_main_head[8], split = "#plate_type\t")[[1]][2]
    instrument_software_version <<- strsplit(ddes_import_main_head[9], split = "#instrument_software_version\t")[[1]][2]
    well_ids <<- unique(ddes_import_main$well_ID)
    
    # Retrieve information in DDES info tables and make available in environment
    channel_id <<- ddes_import_probe$channel_ID
    assay_names <<- ddes_import_assay$assay_ID
    target_id <<- ddes_import_probe$target_ID
    targetchannel_id <<- unlist(strsplit(ddes_import_assay$probe_combination, split= ";"))

    
    # Update progress bar
    incProgress(3/progress_steps, detail = paste("Step", 3, "of", progress_steps))

    # Close progress bar
    
  })
  
}

retrieve_upload_dataset_nio <- function(){
  
  print("start server side upload")

  # Get to the real filenames and not the temp file name
  to <<- file.path(dirname(nio_results_file[['datapath']]), basename(nio_results_file[['name']]))
  file.rename(nio_results_file[['datapath']], to)
  nio_results_file[['datapath']] <<- file.path(dirname(nio_results_file[['datapath']]), basename(nio_results_file[['name']]))
  
  to <<- file.path(dirname(nio_fluo_data_files[['datapath']]), basename(nio_fluo_data_files[['name']]))
  file.rename(nio_fluo_data_files[['datapath']], to)
  nio_fluo_data_files[['datapath']] <<- file.path(dirname(nio_fluo_data_files[['datapath']]), basename(nio_fluo_data_files[['name']]))
  
  to <<- file.path(dirname(nio_exp_details_file[['datapath']]), basename(nio_exp_details_file[['name']]))
  file.rename(nio_exp_details_file[['datapath']], to)
  nio_exp_details_file[['datapath']] <<- file.path(dirname(nio_exp_details_file[['datapath']]), basename(nio_exp_details_file[['name']]))
  
  
  if (is.null(nio_results_file)){return(NULL)}
  if (is.null(nio_fluo_data_files)){return(NULL)}
  if (is.null(nio_exp_details_file)){return(NULL)}
  
  nio_results_fname <<- nio_results_file$datapath
  nio_data_files_fnames <<- nio_fluo_data_files$datapath
  nio_data_base_fnames <<- nio_fluo_data_files$name
  nio_exp_details_fname <<- nio_exp_details_file$datapath
  
  print("Start DDES Conversion for NIO data")

  # 0. Read in exported files from NIO
  
  # Read in RFU files and combine in one
  ruby_wells <- c(paste(LETTERS[1:8], 1, sep=""), paste(LETTERS[1:8], 2, sep=""))
  amplitude_raw_pattern <- paste(paste0(ruby_wells,"_RawData.csv"),collapse = "|")
  amplitude_comp_pattern <- paste(paste0(ruby_wells,"_CompensatedData.csv"),collapse = "|")
  
  amplitude_filenames_raw <<- nio_data_files_fnames[grep(nio_data_files_fnames, pattern = amplitude_raw_pattern)]
  amplitude_filenames_comp <<- nio_data_files_fnames[grep(nio_data_files_fnames, pattern = amplitude_comp_pattern)]
  
  
  # Read in results table
  results_table <- read.csv(file = nio_results_fname, header = TRUE, row.names = NULL)
  names(results_table) <- trimws(names(results_table))
  results_table <- as.data.frame(lapply(results_table,trimws))
  
  
  # Read in experiment file
  exp_details <- read.csv(file = nio_exp_details_fname, header = TRUE, row.names = NULL)
  names(exp_details) <- trimws(names(exp_details))
  exp_details[1,] <- trimws(exp_details[1,])
  
  ## 1. Retrieve general information for DDES headers
  
  instrument <- exp_details$Instrument.Type # Prism3/Prism6/Nio will get this from DDES selection
  plate_type <- exp_details$Chip.Type # Ruby or Saphire will get this from DDES selection?
  instrument_software_version <- "not found in export" # not found, add manually in DDES?
  classification_software_version <- "not found in export" # not found, add manually in DDES?
  classification_method <- "not found in export" # not found, add manually in DDES?
  run_type <- "endpoint" # for Stilla this is endpoint
  runid <<- gsub(x = nio_exp_details_file$name,pattern = "_Experiment_Details.csv", replacement = "") # retrieve from filename
  #run id via grep
  #grep(strsplit(exp_details_base_filename,split="_")[[1]], pattern = "Experiment")-1
  experimentid <<- "NA" # to be added by enduser
  partition_volume_nl <- "not found in export" # what volume do we use?
  well_ids <- results_table$ChamberID # get from chamberid results file
  assay_names <- "" # where to find this?
  target_names <- as.character(exp_details[1,grep(colnames(exp_details),pattern = "name",value = TRUE)]) # get from exp details
  if(length(amplitude_filenames_raw) == 0){
    compensated <<- "yes"
    amplitude_filenames <<- amplitude_filenames_comp} else { compensated <<- "no"
    amplitude_filenames <<- amplitude_filenames_raw}
  
  ## 2. Assay Details file
  
  # Create Probe info based on head file
  probes <- as.data.frame(as.character(exp_details[1,grep(colnames(exp_details),pattern="_name")]))
  probes$target <- as.character(exp_details[1,grep(colnames(exp_details),pattern="_target")])
  colnames(probes) <- c("Target","TargetType")
  probes$name <- paste(probes$Target,probes$TargetType,sep="_")
  probes$mod5 <- probes$Target
  
  
  
  probes <- probes[,c(1,3,2,4)]
  
  ddes_probe_info <- probes
  ddes_probe_info_colnames <- c("target_name","probe_name","channel_ID",
                                "5prime_mod")#,"internal_mod","3prime_mod")
  colnames(ddes_probe_info) <- ddes_probe_info_colnames
  rownames(ddes_probe_info) <- 1:dim(probes)[1]
  
  ddes_probe_info$channel_ID <- c("C","G","R","O","Y")
  # Detect assays on the fly
  #assays <- unique(head_file[,c("Well","Target")])
  #assays <- aggregate(Target ~ Well, data = assays, FUN = function(x) paste(unique(x), collapse = ", "))
  #assays$Assay_number <- match(assays$Target, unique(assays$Target))
  
  #assays$assay_name <- paste0("assay_", assays$Assay_number)
  
  #assays_targets <- unique(assays[,c("assay_name","Target")])
  assays_targets <- ddes_probe_info$target_name
  
  # Create Assay info based on head file and on the fly
  ddes_assay_info_colnames <- c("assay_name","assay_type","target_names",
                                "probe_combination","probe_concentrations_nM")
  ddes_assay_info <- c()
  
  #for(c in 1:dim(assays_targets)[1]){
  
  
  assay_name <- "Rainbow 3"
  assay_type <- "NA" # How to detect this?
  
  #targets_tmp <- strsplit(assays_targets[assays_targets$assay_name==assay_name,]$Target,
  #                        split = ", ")[[1]]
  targets_tmp <- assays_targets
  
  probe_combinations <- ddes_probe_info[which(targets_tmp %in% ddes_probe_info$target_name),]$probe_name
  
  probe_concentrations_nM <- "NA" # How to detect this? Not necessary here?
  
  assay_info_tmp <- c(assay_name, assay_type, paste(target_names, collapse = ";"), 
                      paste(probe_combinations, collapse = ";"), probe_concentrations_nM)
  
  ddes_assay_info <- rbind(ddes_assay_info, assay_info_tmp)
  
  #}
  colnames(ddes_assay_info) <- ddes_assay_info_colnames
  rownames(ddes_assay_info) <- 1:dim(ddes_assay_info)[1]
  ddes_assay_info <- as.data.frame(ddes_assay_info)
  
  
  
  # Construct and export DDES assay details file (not dynamic yet with compensation matrices)
  ddes_assay_details_header <- c("#head",
                                 paste("#run_ID", runid, sep = "\t"),
                                 paste("#experiment_ID", experimentid, sep = "\t"),
                                 "#data",
                                 "#probe_information",
                                 "#assay_information") # "#compensation_matric;assay_name
  
  
  ## 2. Main file
  
  # Construct header
  ddes_main_head <- c("#head",
                        paste("#instrument", instrument, sep = "\t"),
                        paste("#plate_type", plate_type, sep = "\t"),
                        paste("#instrument_software_version", instrument_software_version, sep = "\t"),
                        paste("#classification_software_version", classification_software_version, sep = "\t"),
                        paste("#classification_method", classification_method, sep = "\t"),
                        paste("#run_type", run_type, sep = "\t"), #endpoint/real_time/melt_curve
                        paste("#run_ID", runid, sep = "\t"),
                        paste("#experiment_ID", experimentid, sep = "\t"),
                        paste("#partition_volume_nl", partition_volume_nl, sep = "\t"),
                        "#data")
  
  # construct data table
  ddes_main_colnames <- c("well", "sample_name", "sample_type","target_name", "target_type",
                          "assay_name","sample_fraction_in_mix", "counts_positive", "counts_negative", 
                          "counts_total","threshold_if_threshold")
  
  ddes_main_data <- as.data.table(results_table)
  
  #in one go
  ddes_main_data <- as.data.frame(data.table::melt(ddes_main_data, id.vars = c("ChamberID","SampleName","TotalNumberOfDroplets"), 
                                                   measure.vars = patterns(Concentration="Concentration$",Positive="Positive",
                                                                           Negative="Negative"),
                                                   value.name = c("Concentration","Positive","Negative")
  ))
  
  #separate to retain blue, teal, etc naming
  #ddes_main_data <-as.data.frame(melt(results_table_dt, id.vars = c("ChamberID","SampleName","TotalNumberOfDroplets"), 
  #                      measure.vars = patterns(Concentration="Concentration$"),
  #                      value.name = c("Concentration")
  #))
  
  #ddes_main_data_tmp <- as.data.frame(melt(results_table_dt, id.vars = c("ChamberID","SampleName","TotalNumberOfDroplets"), 
  #     measure.vars = patterns(Positive="Positive",
  #                             Negative="Negative"),
  #     value.name = c("Positive","Negative")
  #))
  #ddes_main_data <- cbind(ddes_main_data,ddes_main_data_tmp)
  
  ddes_main_data$target_type <- "NA" #ask for annotation NTC in sampleName?
  ddes_main_data$sample_type <- "NA" #ask for annotation NTC in sampleName?
  ddes_main_data$sample_fraction_in_mix <- "NA"
  ddes_main_data$assay_name <- "Rainbow 3" # automate by pasting targets to each other?
  ddes_main_data$threshold_if_threshold <- "NA" # automate by pasting targets to each other?
  levels(ddes_main_data$variable) <- c("1"=target_names[1],"2"=target_names[2],
                                       "3"=target_names[3],"4"=target_names[4],"5"=target_names[5])
  
  ddes_main_data <- ddes_main_data[,c(1:2,9,4,8,11,10,6,7,3,12)]
  colnames(ddes_main_data) <- ddes_main_colnames
  
  # GET TO OBJECTS USED BY APP
  
  ddes_import_main_head <<- ddes_main_head
  
  ddes_import_main <- replace(ddes_main_data,ddes_main_data=="NA", "-")
  
  ddes_import_main$well_samplename <- paste(ddes_import_main$well,ddes_import_main$sample_name,
                                            sep="_")
  ddes_import_main <<- ddes_import_main
  ddes_import_assay_head <<- ddes_assay_details_header
  ddes_import_probe <<- ddes_probe_info
  ddes_import_assay <<- ddes_assay_info
  well_ids <<- unique(ddes_import_main$well)
  
  channel_id <<- ddes_import_probe$channel_ID
  assay_names <<- ddes_import_assay$assay_name
  target_names <<- ddes_import_probe$target_name
  
  ddes_import_main_head_df <<- data.frame(do.call(rbind,str_split(ddes_import_main_head, "\t", n = 2)))
  ddes_import_assay_head_df <<- data.frame(do.call(rbind,str_split(ddes_import_assay_head, "\t", n = 2)))
  #ddes_import_main_head <<- data.frame(do.call(rbind,str_split(ddes_import_main_head, " ", n = 2)))
  #ddes_import_assay_head <<- data.frame(do.call(rbind,str_split(ddes_import_assay_head, " ", n = 2)))
  
  
  intensity_files_data_list <- list()
  intensity_files_header_list <- list()
  
  intensity_filenames <- paste0("DDES_", runid, "_", experimentid,
                                "_intensity_", well_ids,".tsv")
  
  # Construct intensity files in DDES format
  ddes_intensity_colnames <- c("partition_ID","channel_ID","FI_endpoint")
  ddes_header <- c("#head", "run_ID", "experiment_ID", "well_ID","assay_name",
                   "compensated","#data")
  
  #compensation_status <- "no" #c("yes","no"), not sure for QIAgen what to report
  
  for(i in 1:length(well_ids)){
    
    #Retrieve assay name
    #assay_name_tmp <- assays[assays$Well==well_ids[d],]$assay_name
    assay_name_tmp <- "Rainbow 3"
    
    #Retrieve amplitudefile
    amplitudefile <- read.csv(file=amplitude_filenames[i], header = TRUE)

    amplitudefile$channel1 <- "C"
    amplitudefile$channel2 <- "G"
    amplitudefile$channel3 <- "R"
    amplitudefile$channel4 <- "O"
    amplitudefile$channel5 <- "Y"   
    
    #Retrieve channel col names
    channel_colnames <- colnames(amplitudefile)[grep(colnames(amplitudefile), pattern="Chan")]
    
    #Construct DDES intensity data
    ddes_intensity_tmp <- as.data.frame(mapply(c,amplitudefile[,c("Chan1_FluoValue","channel1")], 
                                               amplitudefile[,c("Chan2_FluoValue", "channel2")],
                                               amplitudefile[,c("Chan3_FluoValue", "channel3")],
                                               amplitudefile[,c("Chan4_FluoValue", "channel4")],
                                               amplitudefile[,c("Chan5_FluoValue", "channel5")]))
    ddes_intensity_tmp$partition_ID <- rep(c(1:length(amplitudefile$Chan1_FluoValue)),5)
    
    colnames(ddes_intensity_tmp) <- c("FI_endpoint","channel_ID","partition_ID")
    ddes_intensity_tmp_converted <- ddes_intensity_tmp[,c(3,2,1)]
    
    # Construct header lines for each well (could also use ">")
    ddes_header_lines_well <- c("#head",
                                paste("#run_ID", runid, sep = "\t"),
                                paste("#experiment_ID", experimentid, sep = "\t"),
                                paste("#well_ID", well_ids[i], sep = "\t"),
                                paste("#assay_name", assay_name_tmp, sep = "\t"),
                                paste("#compensated", compensated, sep = "\t"),
                                "#data")
    
    # intensity_file_tmp <- ddes_intensity_files[i]
    
    intensity_files_data_list[[i]] <- ddes_intensity_tmp_converted
    names(intensity_files_data_list)[i] <- gsub(intensity_filenames[i], pattern = ".tsv", replacement = "")
    intensity_files_header_list[[i]] <- ddes_header_lines_well
    names(intensity_files_header_list)[i] <- gsub(intensity_filenames[i], pattern = ".tsv", replacement = "")
  }
  intensity_files_data_list <<- intensity_files_data_list
  intensity_files_header_list <<- intensity_files_header_list
  
}

## EOS