################################################################################

#                       DDES Shiny Application: ui                             #

################################################################################


## Created by: Wim Trypsteen
## Debugged by: Emile Roose
## v1.0 June 2025

################################################################################

library(shiny)

shinyUI(
  
  fluidPage(
    
    # Initiate Shiny Feedback
    useShinyFeedback(),

    # Title Panel: DDES Converter & Annotation Software
    titlePanel("DDES Converter & Annotation Software (v1.0)"),
    
    # Layout of the Shiny App with sidebar which takes side and main panels
    sidebarLayout(
      
      # Side bar panels conditioned on the tab
      sidebarPanel(
        
        # First conditional side bar, takes the value of the main tabset panel
        # To do: switch the order: first have main-assay-intensity-download
        # To do: add direct downloadbutton in first panel
        # To do: add dropdown for demo datasets for all platforms (use croco data)
        # To do: check button clicks if not pressed in right order, should not give crash of app
        # To do: integrate upload dataset functions
        # To do: handling missing files, only fluoresence files required
        # To do: integrate upload in stead of 3 separate files per platform
        # To do: check on RFU files if htere are assays/targets in channels not uploaded?
        # To do: think of better sample annotation way (eg.master check boxes controlling per 8)
        # To do: implement starting from only RFU files
        # To do: use popup to select probes as alternative to probe seleciton in probe table (see probe concentrations)
        # To do: check wheb probconcentrations are added for uniqueness
        # To do: when misclicks happen (e.g. save instead of add)
        # To do: if upload files are not in the right upload button or wrong format
        conditionalPanel(
          
          condition = "input.conditionedPanels == 1",
          
          helpText("Explore DDES Data Sets"),
          
          selectInput('ddes_demo_datasets', 
                      'Select DDES demo data set',
                      choices  = c('QIAcuity','Select'),
                      selected = 'Select'
          ),
          
          actionButton("start_demo", label="Start DDES Demo"),
          
          tags$hr(),
          
          helpText("Perform DDES conversion"),
          
          selectInput('dpcr_platform', 
                      'Select dPCR platform',
                      choices  = c('QIAcuity',
                                   #'Nio',
                                   'DDES',
                                   'QX200_Quantasoft',
                                   #'Manual_from_intensity_files',
                                   'Select'),
                      selected = 'Select'
          ),
          
          
          # Conditional file Input based on SelectInput choice from above
          # Conditional 1
          conditionalPanel(
            
            condition = "input.dpcr_platform == 'QX200_Quantasoft'",
            
            fileInput('qx200_head', 
                      'Upload head file',
                      accept=c('text/csv', 
                               'text/comma-separated-values,text/plain', 
                               '.csv'),
                      multiple=FALSE),
            
            fileInput('qx200_amplitude', 
                      'Upload Amplitude files',
                      accept=c('text/csv', 
                               'text/comma-separated-values,text/plain', 
                               '.csv'),
                      multiple=TRUE)
            
          ),
          
          # Conditional 2
          conditionalPanel(
            
            condition = "input.dpcr_platform == 'QIAcuity'",
            
            fileInput('qq_current_results', 
                      'Upload analysis file',
                      accept=c('text/csv', 
                               'text/comma-separated-values,text/plain', 
                               '.csv'),
                      multiple=TRUE),
            
            fileInput('qq_plate_layout', 
                      'Upload plate layout',
                      accept=c('text/csv', 
                               'text/comma-separated-values,text/plain', 
                               '.csv'),
                      multiple=TRUE),
            
            fileInput('qq_RFU', 
                      'Upload RFU files (zipped)',
                      accept=c(".zip"#,
                               #'text/csv', 
                               #'text/comma-separated-values,text/plain', 
                               #'.csv'
                      ),
                      multiple=FALSE)
            
            
          ),
          
          # Conditional 3           
          conditionalPanel(
            
            condition = "input.dpcr_platform == 'Nio'",
            
            fileInput('nio_results', 
                      'Upload results table',
                      accept=c('text/csv', 
                               'text/comma-separated-values,text/plain', 
                               '.csv'),
                      multiple=FALSE),
            
            fileInput('nio_fluo_data', 
                      'Upload fluoresence data files',
                      accept=c('text/csv', 
                               'text/comma-separated-values,text/plain', 
                               '.csv'),
                      multiple=TRUE),
            
            fileInput('nio_exp_details', 
                      'Upload experiment details file',
                      accept=c('text/csv', 
                               'text/comma-separated-values,text/plain', 
                               '.csv'),
                      multiple=FALSE)
            
          ),
        

        # Conditional 4
        conditionalPanel(
            
           condition = "input.dpcr_platform == 'DDES'",
            
           fileInput('ddes_zip_upload', 
                    'Upload DDES files as .ddes or .zip',
                    accept=c(".zip", 
                             ".ddes"),
                    multiple=FALSE)
            
        ),
          
        # Conditional 5
          
          
        tags$hr(),
        
        # Action buttons 
        #checkboxInput("direct_download", label = "Include Direct download", value = FALSE),
        actionButton("start", label="Start DDES Conversion"),
        tags$hr(),
        
        # To do: add download button for direct download
        #actionButton("start_download", label="Start DDES Conversion with Download"),

          
        # End of 1st conditional sidepanel 
        ),
        
        
        # Make 6th conditional panel for intensity file viewing
        conditionalPanel(
        
        condition="input.conditionedPanels == 6",
          
        helpText("Select Intensity File"),
        
        uiOutput("sample_selector"),
          
        actionButton("view_ddes_intensity_file", label="View DDES Intensity")
        
        # show table with detected channels
        
        # End of 6th conditional panel
        ),
        
    # set width of sidebars
    width = 2),

    # End of sideBarPanels
    
    # Main panel section with different tabs 
    mainPanel(
      
      # Create different tabPanels with this set
      tabsetPanel(id = "conditionedPanels",
        

        # Tab 1 content
        tabPanel("DDES Run Info", value=1,
                 
                 # Tab 1 layout initialization
                 fluidRow(
                   
                  h3("Run Information"),
                  uiOutput("helptext_runinfo1"),

                  uiOutput("exp_id_user_ui"),
                  uiOutput("run_id_user_ui"),
                  uiOutput("instrument_user_ui"),
                  uiOutput("instrument_software_version_user_ui"),
                  uiOutput("plate_type_user_ui"),
                 
                  uiOutput("helptext_runinfo2"),
                 
                  uiOutput("update_ddes_main_head")
                 
                 # Close Tab 1 Fluidrow
                 )
                 
        # End of Tab 1 Content
        ),
        
        tabPanel("DDES Probe & Assay Annotation", value=2, 
                 
                 # Tab 2 layout initialization
                 fluidRow(
                   
                 # Create the left view
                 column(4,
                          
                        h3("Probe Information"),
                        #  helpText("DDES Probes"), 
                        #  tags$hr(),
                        uiOutput("helptext_probeinfo1"),
                        textInput("probe_name_tmp", "Probe Name", value = ""),
                          actionButton("add_probe", label="Add Probe"),
                          actionButton("delete_probe", label="Delete Probe(s)"),
                          actionButton("update_ddes_probe", label="Save Probe info"),
                        #  tags$hr(),
                        uiOutput("helptext_probeinfo2"),
                        rHandsontableOutput("ddes_probe2")
                 
                 ),
                 
                 # Create the right view
                 column(8,
                        h3("Assay Information"),
                        
                        #  helpText("DDES Assays"), 
                        #  tags$hr(),
                        uiOutput("helptext_assayinfo1"),
                          textInput("assay_name_tmp", "Assay Name", value = ""),
                          actionButton("add_assay", label="Add Assay"),
                          actionButton("delete_assay", label="Delete Assay(s)"),
                          actionButton("update_ddes_assay", label="Save Assay info"),
                        uiOutput("helptext_assayinfo2"),
                        rHandsontableOutput("ddes_assay2")
                        
                 )
                 
                 )

                 
                 # End of Tab 2     
        ),
        
        tabPanel("DDES Sample Annotation", value=3, 
                 
                 h3("DDES Sample Annotation"),
                 #helpText("Select Assay"),
                 uiOutput("helptext_annotinfo1"),
                 uiOutput("assay_selector"),
                 
                 actionButton("apply_assay_to_well", label="Apply Assay"),
                 actionButton("update_sample_id", label="Update Sample ID(s)"),
                 uiOutput("helptext_annotinfo2"),
                 rHandsontableOutput("ddes_sample_annotation")
                 
                 # End of Tab 3     
        ),
        
        tabPanel("DDES Main", value=4, 
                 
                 h3("DDES MAIN HEADER"),
                 tableOutput("ddes_main_head"),

                 h3("DDES MAIN OVERVIEW TARGETS PER WELL"),
                 tableOutput("ddes_main")
                 
        # End of Tab 4     
        ),
        
        # Tab 5 content
        
        tabPanel("DDES Assays", value=5,
                 
                 h3("DDES ASSAYS HEADER"),
                 tableOutput("ddes_assay_head"),
                 
                 h3("ASSAY INFORMATION"),
                 tableOutput("ddes_assay")

        # End of Tab 5
        ),
        
        # Tab 6 content
        tabPanel("DDES Intensity Files", value=6,
                 
                 h3("DDES INTENSITY HEADER OVERVIEW SELECTED SAMPLE"),
                 tableOutput("ddes_intensity_head"),

                 h3("DDES INTENSITY DATA OVERVIEW SELECTED SAMPLE (first 10 rows)"),
                 tableOutput("ddes_intensity_data")
                 
        # End of Tab 6
        ),
        
        # Tab 7 content
        tabPanel("Download DDES Files", value=7,
                 
                 h3("Download DDES files package"),
                 #downloadButton('downloadDDES', 'Download DDES FILES')

                 actionButton("validate_download","Download DDES Files")#, class="btn-success btn-lg")
                 #h3("Download DDES Main File"),
                 #downloadButton('downloadMain', 'Download DDES MAIN'),
                 #h3("Download DDES Assay Details File"),
                 #downloadButton('downloadAssay', 'Download DDES ASSAY'),
                 #h3("Download DDES Intensity Files"),
                 #downloadButton('downloadIntensities', 'Download DDES INTENSITIES')
                 
        # End of Tab 7
        ), 
        
        # Tab 8 content
        tabPanel("How To Use", value=8,
                 
                 h4("How To Use this DDES Software Tool"),
                 p("This application works with consecutive tabs"),
                 p("Please work your way through from left to right"),
                 p("You can switch between tabs and inspect changes you made"),
                 
                 h4("1. Data Input (Side Panel"),
                 p("a) Explore a DDES Demo data set: Select DDES Demo data set and Click Start DDES Demo"),
                 p("b) Upload your DDES/dPCR platform files, based on your choice, DDES/platform specific files are needed for upload"),
                 p("   Start by clicking Start DDES Conversion"),

                 h4("2. DDES Run Info Tab"),
                 p("Here you can inspect metadata automatically retrieved from the platform output files."),
                 p("If certain information is not available, NA wil show but you can change this via a dropdown selection or edit the text"),
                 p("If changes are made, please click the Save button to store these. On the right bottom an update screen wil show if succesful"),

                 h4("3. DDES Probe & Assay Annotation Tab"),
                 p("Here you can inspect target, channel and assays that are automatically retrieved from the platform output files."),
                 p("You can edit this information by adding, deleting or updating probe or assay information"),
                 p("For Probes"),
                 p("To add probe: Type name, click Add and select desired channel"),
                 p("To delete probe: Select probe in the selector column and click Delete. Probes in use will not be deleted"),
                 p("To change info target ID: Edit the name in the table and click Save. Only unique target-channel combinations are allowed."),
                 p("To provide a color combination (multiple probes for one target), use the same target name with other channel ID. eg. PSI GREEN, PSI YELLOW"),
                 p("For Assays"),
                 p("To add assay: Type name, select probes for the assay in the selector column of the probe table and click Add"),
                 p("To delete assay: Select assay in the selector column and click Delete. Assays in use will not be deleted"),
                 p("To change info assay ID: Edit the name in the table and click Save. Only unique assay names are allowed."),
                 p("To change probe concentrations: click the field in the table and provide the concentration info in the pop-up window"),
                 p(""),
                 p("If changes are made, please click the Save button to store these. On the right bottom an update screen wil show if succesful"),
                 
                 h4("4. Sample Annotation Tab"),
                 p("Here you can inspect sample annotation with detected assays that are automatically retrieved from the platform output files."),
                 p("You can edit the assay information and sample naming"),
                 p("For updating assay information"),
                 p("Select assay from the dropdown, select the wells in the selector column and click Apply Assay"),
                 p("For changing sample naming"),
                 p("Fill in, copy-paste or use fill grip to annotate samples"),
                 p(""),
                 p("If changes are made, please click the Save button to store these. On the right bottom an update screen wil show if succesful"),
                 
                 h4("5. DDES Main, Assays and Intensity Files"),
                 p("In these tabs the specific DDES format files are shown. You can inspect these at any time during the annotation work. These are not editable."),
                 
                 h4("6. Data Download"),
                 p("Here you can download your DDES Data as .ddes archive"),
                 
                 h4("7. Contact"),
                 p("Hope you enjoy this Shiny application for DDES file conversion and editing"),
                 p("For questions contact wim.trypsteen@ugent.be or the DIGPCR UGent Center")
        
                 # End of Tab 8        
        )

      # close tabset panel
      ),
      
   # set width of main panel and close
   width = 10
   )
   
 # Close sidebarlayout
 )
 
 # close ui and fluid page
)
)