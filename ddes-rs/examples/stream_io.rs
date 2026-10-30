//! Seekable I/O streams, not row-by-row or bounded-memory processing.
//! Run with an existing ZIP/.ddes archive (not a directory).

use std::{
    fs::File,
    io::{BufReader, Cursor},
};

use ddes::Dataset;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        return Err("usage: cargo run --example stream_io -- <input.zip-or-ddes>".into());
    }

    // BufReader<File> implements Read + Seek. The full Dataset is still loaded.
    let input = BufReader::new(File::open(&args[0])?);
    let dataset = Dataset::read_zip(input)?;

    // Cursor<Vec<u8>> implements Write + Seek. write_zip finishes the ZIP and
    // returns the writer, so its bytes can be passed to another library.
    // A File also works here when the output should go directly to disk.
    let output = dataset.write_zip(Cursor::new(Vec::new()))?;
    let bytes = output.into_inner();
    println!(
        "Serialized dataset (main records: {}, assay records: {}, intensity files: {}) into {} ZIP bytes",
        dataset.main.records.len(),
        dataset.assays.records.len(),
        dataset.intensities.len(),
        bytes.len(),
    );

    // Use a new cursor positioned at the beginning. Reading borrowed bytes
    // avoids copying the ZIP buffer, but parsing allocates another Dataset.
    let reloaded = Dataset::read_zip(Cursor::new(bytes.as_slice()))?;
    if reloaded != dataset {
        return Err("stream I/O round-trip changed the dataset".into());
    }
    println!("Verified semantic round trip through seekable I/O streams");
    Ok(())
}
