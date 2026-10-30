//! R component tables and archive I/O for DDES v1.0.0.

use extendr_api::prelude::*;

mod archive;
mod assays;
mod convert;
mod intensity;
mod main_file;

type Result<T> = std::result::Result<T, String>;

extendr_module! {
    mod ddes;
    use archive;
    use assays;
    use intensity;
    use main_file;
}
