//! Read a TSV directory or ZIP/.ddes archive and verify a normalized round trip.
//! Loads the dataset into memory. The output path must not already exist.

use std::path::PathBuf;

use ddes::Dataset;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: cargo run --example roundtrip -- <directory-or-zip> <new-output.ddes>".into(),
        );
    }
    let input = PathBuf::from(&args[0]);
    let output = PathBuf::from(&args[1]);
    let dataset = Dataset::read_path(&input)?;
    println!(
        "Experiment {}, run {}: main records: {}, assay records: {}, intensity files: {}, intensity records: {}",
        dataset.main.metadata.experiment_id,
        dataset.main.metadata.run_id,
        dataset.main.records.len(),
        dataset.assays.records.len(),
        dataset.intensities.len(),
        dataset
            .intensities
            .iter()
            .map(|file| file.records.len())
            .sum::<usize>()
    );
    if let Some(record) = dataset.main.records.first() {
        println!(
            "First result: well {}, target {}, positive count {:?}",
            record.well_id, record.target_id, record.counts_positive
        );
    }
    dataset.write_path(&output)?;
    let reloaded = Dataset::read_path(&output)?;
    if dataset != reloaded {
        return Err("round-trip changed the dataset".into());
    }
    println!("Verified semantic round trip: {}", output.display());
    Ok(())
}
