use std::io::{Cursor, Write};

use ddes::{AssaysFile, Dataset, Error, IntensityFile, MainFile, MainMetadata};
use zip::{ZipWriter, write::SimpleFileOptions};

const MAIN: &str = include_str!("fixtures/DDES_expID_run_202506301127_main.tsv");
const ASSAYS: &str = include_str!("fixtures/DDES_expID_run_202506301127_assays.tsv");
const INTENSITY: &str = include_str!("fixtures/DDES_expID_run_202506301127_intensity_A1.tsv");

fn fixture() -> Dataset {
    Dataset::read_path(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).unwrap()
}

fn zip(files: &[(&str, &str)]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, contents) in files {
        writer
            .start_file(*name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(contents.as_bytes()).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn read_zip(bytes: Vec<u8>) -> ddes::Result<Dataset> {
    Dataset::read_zip(Cursor::new(bytes))
}

#[test]
fn shared_binding_contract() {
    let dataset =
        Dataset::read_path(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/contract")).unwrap();
    let rows = &dataset.main.records;
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].counts_positive, Some(9_007_199_254_740_993));
    assert_eq!(rows[1].counts_positive, Some(u64::MAX));
    assert_eq!(rows[2].counts_positive, None);
    assert_eq!(rows[0].counts_negative, Some(0));
    assert_eq!(rows[1].counts_negative, None);
    assert_eq!(rows[2].counts_negative, Some(17));
    assert_eq!(rows[0].concentration_reaction_cp_ul, Some(0.0));
    assert_eq!(rows[1].concentration_reaction_cp_ul, None);
    assert_eq!(rows[2].concentration_reaction_cp_ul, Some(1.25));
    assert_eq!(rows[0].threshold_if_threshold, Some(-1.5));
    assert_eq!(rows[1].threshold_if_threshold, None);
    assert_eq!(rows[2].threshold_if_threshold, Some(0.0));
    assert_eq!(rows[0].sample_id, "001");
    assert_eq!(rows[2].sample_id, "µ-sample");
    assert_eq!(dataset.assays.records[0].probe_concentrations_nm, "NA");
    assert_eq!(
        dataset.assays.records[1].probe_concentrations_nm,
        "125;NA;0"
    );
    let records = &dataset.intensities[0].records;
    assert_eq!(records[0].partition_id, "001");
    assert_eq!(records[0].fi_endpoint, -0.5);
    assert_eq!(records[1].fi_endpoint, 0.0);
    assert_eq!(records[2].fi_endpoint, 100.25);
    assert_eq!(records[3].fi_endpoint, 2.5);
    let bytes = dataset
        .write_zip(Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    assert_eq!(read_zip(bytes).unwrap(), dataset);
}

#[test]
fn reference_values_and_archive_roundtrip() {
    let dataset = fixture();
    assert_eq!(dataset.main.metadata.ddes_version, "1.0");
    assert_eq!(dataset.main.metadata.instrument, "QIAcuity Four");
    assert_eq!(dataset.main.records.len(), 3);
    let first = &dataset.main.records[0];
    assert_eq!((&*first.well_id, &*first.target_id), ("A1", "PSI"));
    assert_eq!(first.counts_positive, Some(4236));
    assert_eq!(first.counts_negative, Some(21173));
    assert_eq!(first.concentration_reaction_cp_ul, Some(241.9));
    assert_eq!(dataset.main.records[2].threshold_if_threshold, Some(42.23));
    assert_eq!(dataset.assays.records[0].assay_id, first.assay_id);
    assert_eq!(
        dataset.assays.records[0].probe_combination,
        "PSI.GREEN;ENV.YELLOW;GAG.CRIMSON"
    );
    assert_eq!(
        dataset.assays.records[0].probe_concentrations_nm,
        "125;125;125"
    );
    let intensity = &dataset.intensities[0];
    assert_eq!(intensity.metadata.well_id, "A1");
    assert_eq!(intensity.records.len(), 9);
    assert_eq!(intensity.records[0].partition_id, "9");
    assert_eq!(intensity.records[0].channel_id, "CRIMSON");
    assert_eq!(intensity.records[0].fi_endpoint, 34.65);
    assert_eq!(intensity.records[3].fi_endpoint, 14.71);
    let bytes = dataset
        .write_zip(Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    assert_eq!(read_zip(bytes).unwrap(), dataset);
    // Serde is part of the public model, not only an internal CSV detail.
    let json = serde_json::to_string(&dataset).unwrap();
    assert_eq!(serde_json::from_str::<Dataset>(&json).unwrap(), dataset);
    let wire: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(wire["main"]["records"][0]["counts_positive"], "4236");
    assert_eq!(wire["main"]["records"][0]["counts_negative"], "21173");
    assert_eq!(
        wire["main"]["records"][0]["concentration_reaction_cp_µL"],
        "241.9"
    );
    assert_eq!(
        wire["main"]["records"][0]["threshold_if_threshold"],
        "18.88"
    );
    assert_eq!(
        wire["intensities"][0]["records"][0],
        serde_json::json!({"partition_ID": "9", "channel_ID": "CRIMSON", "FI_endpoint": 34.65})
    );
    assert_eq!(
        wire["intensities"][0]["metadata"],
        serde_json::json!({
            "DDES_version": "1.0", "datetime": "202506301127", "experiment_ID": "expID",
            "run_ID": "5160815c-092d-49ea-946c-cf7bcb49432a", "DDES_type": "intensity",
            "well_ID": "A1", "assay_ID": "HIV Triplex"
        })
    );

    // A well name ending in a role must not turn an intensity file into main.
    let mut renamed = dataset;
    renamed.intensities[0].metadata.well_id = "control_main".into();
    for row in &mut renamed.main.records {
        row.well_id = "control_main".into();
    }
    let bytes = renamed
        .write_zip(Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    assert_eq!(read_zip(bytes).unwrap(), renamed);
}

#[test]
fn finite_intensities_normalize_without_dropping_rows() {
    let input = INTENSITY
        .replace("1.0\n", "1.0.0\n")
        .replace("202506301127", "unspecified date")
        .replace("9\tCRIMSON\t34.65", "0009\t紫\t0")
        .replace("10\tCRIMSON\t60.84", "10\tCRIMSON\t-12.75")
        .replace("11\tCRIMSON\t62.14", "11\tCRIMSON\t 1.25e2 ")
        .replace('\n', "\r\n");
    let file = IntensityFile::from_tsv("intensity.tsv", input.as_bytes()).unwrap();
    assert_eq!(file.records.len(), 9);
    assert_eq!(file.records[0].partition_id, "0009");
    assert_eq!(file.records[0].channel_id, "紫");
    assert_eq!(file.records[0].fi_endpoint, 0.0);
    assert_eq!(file.records[1].fi_endpoint, -12.75);
    assert_eq!(file.records[2].fi_endpoint, 125.0);
    let normalized = file.to_tsv("intensity.tsv").unwrap();
    assert!(!normalized.contains(&b'\r'));
    assert_eq!(
        IntensityFile::from_tsv("intensity.tsv", &normalized).unwrap(),
        file
    );
    for token in ["", "NA", "-", "NaN", "Inf", "-Inf", "1e400", "text"] {
        let input = INTENSITY.replacen("34.65", token, 1);
        let error = IntensityFile::from_tsv("bad.tsv", input.as_bytes()).unwrap_err();
        assert!(
            matches!(error, Error::Format { line: 11, .. }),
            "{token}: {error}"
        );
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut dataset = fixture();
        dataset.intensities[0].records[0].fi_endpoint = value;
        assert!(dataset.intensities[0].to_tsv("bad.tsv").is_err());
        assert!(dataset.write_zip(Cursor::new(Vec::new())).is_err());
        // JSON's default f64 serializer would silently turn these into null.
        assert!(serde_json::to_string(&dataset.intensities[0]).is_err());
    }
    let row = serde_json::json!({
        "partition_ID": "0009", "channel_ID": "紫", "FI_endpoint": -12.75
    });
    let parsed: ddes::IntensityRecord = serde_json::from_value(row.clone()).unwrap();
    assert_eq!(parsed.fi_endpoint, -12.75);
    assert_eq!(serde_json::to_value(&parsed).unwrap(), row);
    for value in [serde_json::Value::Null, serde_json::json!("34.65")] {
        let mut invalid = row.clone();
        invalid["FI_endpoint"] = value;
        assert!(serde_json::from_value::<ddes::IntensityRecord>(invalid).is_err());
    }
}

#[test]
fn main_numeric_domains_and_missing_values() {
    for (cells, expected) in [
        ("0\tNA\t0\t-2.5", (Some(0), None, Some(0.0), Some(-2.5))),
        ("NA\t0\tNA\tNA", (None, Some(0), None, None)),
        ("NA\tNA\tNA\tNA", (None, None, None, None)),
        (
            "+001\t 002 \t 0 \t NA ",
            (Some(1), Some(2), Some(0.0), None),
        ),
        (
            "18446744073709551615\t9007199254740993\t1.25e2\t0",
            (
                Some(u64::MAX),
                Some(9007199254740993),
                Some(125.0),
                Some(0.0),
            ),
        ),
    ] {
        let input = MAIN.replacen("4236\t21173\t241.9\t18.88", cells, 1);
        let file = MainFile::from_tsv("main.tsv", input.as_bytes()).unwrap();
        let row = &file.records[0];
        assert_eq!(
            (
                row.counts_positive,
                row.counts_negative,
                row.concentration_reaction_cp_ul,
                row.threshold_if_threshold
            ),
            expected
        );
        let output = file.to_tsv("main.tsv").unwrap();
        assert_eq!(MainFile::from_tsv("main.tsv", &output).unwrap(), file);
        let json = serde_json::to_string(&file).unwrap();
        assert_eq!(serde_json::from_str::<MainFile>(&json).unwrap(), file);
    }
    let mut row = fixture().main.records.remove(0);
    row.counts_positive = None;
    let mut wire = serde_json::to_value(&row).unwrap();
    assert_eq!(wire["counts_positive"], "NA");
    for value in [serde_json::Value::Null, serde_json::json!(4236)] {
        wire["counts_positive"] = value;
        assert!(serde_json::from_value::<ddes::MainRecord>(wire.clone()).is_err());
    }
    // Try every field so an accidentally omitted serde codec cannot pass.
    for (index, invalid) in [
        (
            0,
            vec!["", "-", "-1", "1.5", "1e2", "18446744073709551616", "NaN"],
        ),
        (1, vec!["", "-1", "1.5", "18446744073709551616"]),
        (2, vec!["", "-", "-0.1", "NaN", "Inf", "1e400"]),
        (3, vec!["", "-", "NaN", "Inf", "-Inf", "1e400"]),
    ] {
        for token in invalid {
            let mut cells = ["3", "17", "0.5", "-2"];
            cells[index] = token;
            let input = MAIN.replacen("4236\t21173\t241.9\t18.88", &cells.join("\t"), 1);
            assert!(
                MainFile::from_tsv("bad.tsv", input.as_bytes()).is_err(),
                "field {index}: {token}"
            );
        }
    }
    for value in [-0.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut file = fixture().main;
        file.records[0].concentration_reaction_cp_ul = Some(value);
        assert!(file.to_tsv("bad.tsv").is_err());
        file.records[0].concentration_reaction_cp_ul = None;
        file.records[0].threshold_if_threshold = Some(value);
        assert_eq!(file.to_tsv("threshold.tsv").is_ok(), value.is_finite());
    }
}

#[test]
fn probe_concentrations_preserve_missing_list_and_entry_positions() {
    for token in [
        "NA",
        "NA;NA",
        "125;NA;0.5",
        "0;2.5e2;NA",
        "125",
        " NA ",
        " 125 ; NA ; 0 ",
    ] {
        let input = ASSAYS.replace("125;125;125", token);
        let file = AssaysFile::from_tsv("assays.tsv", input.as_bytes()).unwrap();
        assert_eq!(file.records[0].probe_concentrations_nm, token);
        assert_eq!(
            AssaysFile::from_tsv("assays.tsv", &file.to_tsv("assays.tsv").unwrap()).unwrap(),
            file
        );
    }
    for token in [
        "",
        "-",
        "na",
        "125;;125",
        ";125",
        "125;",
        "125;-1;125",
        "NaN",
        "125;Inf;NA",
        "1e400",
    ] {
        let input = ASSAYS.replace("125;125;125", token);
        assert!(
            AssaysFile::from_tsv("bad.tsv", input.as_bytes()).is_err(),
            "{token}"
        );
        let mut file = fixture().assays;
        file.records[0].probe_concentrations_nm = token.into();
        assert!(file.to_tsv("bad.tsv").is_err(), "{token}");
    }
}

#[test]
fn reordered_columns_and_quoted_identifiers_survive() {
    let header = INTENSITY.split_once("partition_ID").unwrap().0;
    let input = format!(
        "{header}FI_endpoint\tchannel_ID\tpartition_ID\n 1.25e2 \t\" O'Brien #1 \"\t\" 0009 \"\n"
    );
    let file = IntensityFile::from_tsv("intensity.tsv", input.as_bytes()).unwrap();
    assert_eq!(file.records[0].channel_id, " O'Brien #1 ");
    assert_eq!(file.records[0].partition_id, " 0009 ");
    assert_eq!(file.records[0].fi_endpoint, 125.0);
    assert_eq!(
        IntensityFile::from_tsv("out.tsv", &file.to_tsv("out.tsv").unwrap()).unwrap(),
        file
    );
    let mut main = fixture().main;
    main.records[0].sample_id = "a \"quoted\" sample\twith\na newline".into();
    main.metadata.instrument = "Lab \"instrument\" #1".into();
    let path = std::path::PathBuf::from("not-opened.tsv");
    assert_eq!(
        MainFile::from_tsv(&path, &main.to_tsv(&path).unwrap()).unwrap(),
        main
    );
}

#[test]
fn malformed_tables_report_location_and_extensions_are_not_discarded() {
    let malformed = MAIN.replacen("4236\t21173", "4236", 1);
    let error = MainFile::from_tsv("bad.tsv", malformed.as_bytes()).unwrap_err();
    assert!(
        matches!(error, Error::Format { ref file, line: 12, .. } if file == "bad.tsv"),
        "{error}"
    );
    for input in [
        MAIN.replace("well_ID\tsample_ID", "well_ID\twell_ID"),
        MAIN.replace("#data\n", ""),
        MAIN.replace("#data\n", "#run_ID\tduplicate\n#data\n"),
        MAIN.replace("#data\n", "#operator\tZoë\n#data\n"),
        MAIN.replace("#DDES_type\tmain", "#DDES_type\tintensity"),
        MAIN.replace("#instrument\tQIAcuity Four", "#instrument\tQIAcuity\tFour"),
        MAIN.replace("target_ID", "unexpected_column"),
    ] {
        assert!(MainFile::from_tsv("bad.tsv", input.as_bytes()).is_err());
    }
    let missing = MAIN.replace("#run_ID\t5160815c-092d-49ea-946c-cf7bcb49432a\n", "");
    assert!(
        MainFile::from_tsv("bad.tsv", missing.as_bytes())
            .unwrap_err()
            .to_string()
            .contains("run_ID")
    );
    assert!(
        read_zip(zip(&[
            ("main.tsv", MAIN),
            ("assays.tsv", ASSAYS),
            ("notes.tsv", "extra data")
        ]))
        .is_err()
    );
}

#[test]
fn unsafe_paths_and_duplicate_components_are_rejected() {
    for name in [
        "../main.tsv",
        "dir/main.tsv",
        "/main.tsv",
        "C:\\main.tsv",
        "dir\\main.tsv",
        "CON.tsv",
    ] {
        assert!(
            read_zip(zip(&[(name, MAIN), ("assays.tsv", ASSAYS)])).is_err(),
            "{name}"
        );
    }
    for extra in [
        ("other_main.tsv", MAIN),
        ("MAIN.tsv", MAIN),
        ("other_assay.tsv", ASSAYS),
        ("other_intensity_B1.tsv", INTENSITY),
    ] {
        assert!(
            read_zip(zip(&[
                ("main.tsv", MAIN),
                ("assays.tsv", ASSAYS),
                ("intensity_A1.tsv", INTENSITY),
                extra
            ]))
            .is_err()
        );
    }
    assert!(read_zip(zip(&[("assays.tsv", ASSAYS)])).is_err());
    // A genuinely duplicated ZIP entry, including the same name in the central directory.
    let mut bytes = zip(&[
        ("main.tsv", MAIN),
        ("xxxx.tsv", MAIN),
        ("assays.tsv", ASSAYS),
    ]);
    let positions: Vec<_> = bytes
        .windows(8)
        .enumerate()
        .filter_map(|(i, s)| (s == b"xxxx.tsv").then_some(i))
        .collect();
    for position in positions {
        bytes[position..position + 8].copy_from_slice(b"main.tsv");
    }
    assert!(read_zip(bytes).is_err());
}

#[test]
fn public_construction_empty_tables_and_safe_file_writes() {
    let original = fixture();
    let mut constructed = Dataset {
        main: MainFile {
            metadata: original.main.metadata.clone(),
            records: Vec::new(),
        },
        assays: original.assays.clone(),
        intensities: Vec::new(),
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("résultats.ddes");
    constructed.write_path(&path).unwrap();
    assert_eq!(Dataset::read_path(&path).unwrap(), constructed);
    assert!(constructed.write_path(&path).is_err());
    assert_eq!(Dataset::read_path(&path).unwrap(), constructed);
    constructed.main.metadata.run_id = "../escape".into();
    let invalid = path.with_file_name("invalid.ddes");
    assert!(constructed.write_path(&invalid).is_err());
    assert!(!invalid.exists());
    constructed.main.metadata.run_id = "run\ninvalid header".into();
    assert!(constructed.write_zip(Cursor::new(Vec::new())).is_err());
    constructed.main.metadata.run_id = original.main.metadata.run_id.clone();
    constructed.main.metadata.ddes_type = "intensity".into();
    assert!(matches!(
        constructed.main.to_tsv("main.tsv"),
        Err(Error::Format { field, .. }) if field == "DDES_type"
    ));

    // All three schemas must survive serialization even without a first record.
    let mut empty = original;
    empty.main.records.clear();
    empty.assays.records.clear();
    for file in &mut empty.intensities {
        file.records.clear();
    }
    let bytes = empty
        .write_zip(Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    assert_eq!(read_zip(bytes).unwrap(), empty);

    // An empty table must not bypass column validation because there are no rows.
    let tsv = String::from_utf8(empty.main.to_tsv("empty.tsv").unwrap()).unwrap();
    let invalid = tsv.replace("target_ID", "unexpected_column");
    assert!(MainFile::from_tsv("empty.tsv", invalid.as_bytes()).is_err());
}

#[test]
fn generated_files_in_native_paths_roundtrip() {
    let temp = tempfile::tempdir().unwrap();
    // Linux permits non-UTF-8 names; macOS filesystems may reject their creation.
    #[cfg(target_os = "linux")]
    let (directory_name, archive_name) = {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};
        (
            OsString::from_vec(b"input-\xff".to_vec()),
            OsString::from_vec(b"output-\xfe.ddes".to_vec()),
        )
    };
    #[cfg(not(target_os = "linux"))]
    let (directory_name, archive_name) = ("données", "résultats.ddes");
    let directory = temp.path().join(directory_name);
    std::fs::create_dir(&directory).unwrap();

    // Construct inputs independently of the Rust writer and checked-in R fixtures.
    let header = "#header\n#DDES_version\t1.0.0\n#datetime\tunspecified\n\
                  #experiment_ID\texp-created\n#run_ID\trun-created\n";
    let main = format!(
        "{header}#DDES_type\tmain\n#instrument\tTest instrument\n\
         #plate_type\tCustom plate\n#instrument_software_version\tdev\n#data\n\
         well_ID\tsample_ID\tassay_ID\ttarget_ID\tcounts_positive\tcounts_negative\tconcentration_reaction_cp_µL\tthreshold_if_threshold\n\
         B2\t\"sample \"\"β\"\"\t#1\"\t007\tTarget\t0\t17\tNA\tNA\n"
    );
    let assays = format!(
        "{header}#DDES_type\tassay\n#data\n#assay_information\n\
         assay_ID\tprobe_combination\tprobe_concentrations_nM\n\
         007\tTarget.GREEN\t125\n"
    );
    let intensity = format!(
        "{header}#DDES_type\tintensity\n#well_ID\tB2\n#assay_ID\t007\n#data\n\
         partition_ID\tchannel_ID\tFI_endpoint\n\
         0001\tGREEN\t0\n0002\tGREEN\t1.25\n0003\tGREEN\t-12.75\n"
    );
    for (name, text) in [
        ("main.tsv", main),
        ("assays.tsv", assays),
        ("intensity_B2.tsv", intensity),
    ] {
        std::fs::write(directory.join(name), text).unwrap();
    }

    let dataset = Dataset::read_path(&directory).unwrap();
    assert_eq!(
        dataset.main.metadata,
        MainMetadata {
            ddes_version: "1.0.0".into(),
            datetime: "unspecified".into(),
            experiment_id: "exp-created".into(),
            run_id: "run-created".into(),
            ddes_type: "main".into(),
            instrument: "Test instrument".into(),
            plate_type: "Custom plate".into(),
            instrument_software_version: "dev".into(),
        }
    );
    assert_eq!(dataset.main.records.len(), 1);
    let row = &dataset.main.records[0];
    assert_eq!(
        (
            &*row.well_id,
            &*row.sample_id,
            &*row.assay_id,
            &*row.target_id
        ),
        ("B2", "sample \"β\"\t#1", "007", "Target")
    );
    assert_eq!(row.counts_positive, Some(0));
    assert_eq!(row.counts_negative, Some(17));
    assert_eq!(row.concentration_reaction_cp_ul, None);
    assert_eq!(row.threshold_if_threshold, None);
    assert_eq!(dataset.assays.metadata.ddes_type, "assay");
    assert_eq!(dataset.assays.records.len(), 1);
    let assay = &dataset.assays.records[0];
    assert_eq!(
        (
            &*assay.assay_id,
            &*assay.probe_combination,
            &*assay.probe_concentrations_nm
        ),
        ("007", "Target.GREEN", "125")
    );
    assert_eq!(dataset.intensities.len(), 1);
    let intensity = &dataset.intensities[0];
    assert_eq!(
        (&*intensity.metadata.well_id, &*intensity.metadata.assay_id),
        ("B2", "007")
    );
    assert_eq!(
        intensity
            .records
            .iter()
            .map(|row| (
                row.partition_id.as_str(),
                row.channel_id.as_str(),
                row.fi_endpoint
            ))
            .collect::<Vec<_>>(),
        vec![
            ("0001", "GREEN", 0.0),
            ("0002", "GREEN", 1.25),
            ("0003", "GREEN", -12.75),
        ]
    );
    let output = temp.path().join(archive_name);
    dataset.write_path(&output).unwrap();
    assert_eq!(Dataset::read_path(&output).unwrap(), dataset);
}
