library(shiny)
library(rhandsontable)
library(ddes)

source("annotations.R", local = TRUE)
source("converters.R", local = TRUE)
source("interface.R", local = TRUE)
source("session.R", local = TRUE)

# Shiny evaluates app.R with the application directory as its working directory.
demo_path <- normalizePath("../../../archive/demo_data/QIAcuity", mustWork = TRUE)
options(shiny.maxRequestSize = 50 * 1024^2)
shinyApp(ddes_ui(), ddes_server(demo_path))
