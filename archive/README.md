# Original R/Shiny application

This directory preserves the original DDES converter and annotation application
from [revision 8776b8a](https://github.com/digpcr/DDES/commit/8776b8a71278bd1e17b3599f2ae4bfe29c8f065a).
The R source, demo datasets, and `rsconnect/` deployment metadata are unchanged;
only their location has moved. This is not an R binding to the Rust core.

## Run locally

The app requires R and these packages: `shiny`, `rhandsontable`, `stringr`, `zip`,
`reshape2`, `data.table`, `shinyFeedback`, and `shinyalert`.

From the repository root, run:

```sh
Rscript -e 'shiny::runApp("archive")'
```

Shiny runs the app with this directory as its working directory, preserving its
relative `demo_data/` paths. Keep the app and its assets together.

The `rsconnect/` files describe the existing deployment; moving them does not
update or redeploy the hosted app.
