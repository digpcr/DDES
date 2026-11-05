//! Shared UTF-8, metadata, and TSV table parsing and encoding.

use csv::{QuoteStyle, ReaderBuilder, StringRecord, WriterBuilder};
use serde::{Serialize, de::DeserializeOwned};

use crate::{Error, Result};

/// Validate text encoding and remove a leading BOM before header parsing.
pub(crate) fn validated_input<'a>(file: &str, input: &'a [u8]) -> Result<&'a [u8]> {
    let text = std::str::from_utf8(input)
        .map_err(|e| Error::format(file, 0, "encoding", e.to_string()))?
        .trim_start_matches('\u{feff}');
    if text.contains('\0') {
        return Err(Error::unsupported(
            file,
            "NUL bytes cannot be represented in R text tables",
        ));
    }
    Ok(text.as_bytes())
}

/// Split one metadata line; file modules decide which lines belong to the header.
pub(crate) fn metadata_field(
    file: &str,
    line_number: usize,
    line: &str,
) -> Result<(String, String)> {
    let (key, value) = line
        .strip_prefix('#')
        .and_then(|s| s.split_once('\t'))
        .ok_or_else(|| {
            Error::format(
                file,
                line_number as u64,
                "header",
                "unsupported header line",
            )
        })?;
    if value.contains(['\t', '\r']) {
        return Err(Error::format(
            file,
            line_number as u64,
            key,
            "metadata must fit on a single key/value line",
        ));
    }
    Ok((key.to_owned(), value.to_owned()))
}

/// Read the ordinary TSV table after a file module has consumed its header.
pub(crate) fn read_records<T: DeserializeOwned>(
    file: &str,
    input: &[u8],
    lines: usize,
    trim_columns: &[&str],
) -> Result<Vec<T>> {
    let table_error = |error: csv::Error| {
        // CSV positions precede skipped input, including the LF half of CRLF.
        // Count only leading empty/comment lines, never newlines inside a row.
        let skipped_lines = error.position().map_or(0, |position| {
            input[position.byte() as usize..]
                .split_inclusive(|&byte| byte == b'\n')
                .take_while(|line| {
                    line.starts_with(b"#") || line.iter().all(|byte| matches!(byte, b'\r' | b'\n'))
                })
                .filter(|line| line.ends_with(b"\n"))
                .count()
        });
        csv_error(file, lines + skipped_lines, error)
    };
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .comment(Some(b'#'))
        .from_reader(input);
    let columns = reader.headers().map_err(table_error)?.clone();
    // Check even an empty table: deserializing rows alone cannot check its header.
    let expected_columns = field_names::<T>();
    for (index, column) in columns.iter().enumerate() {
        if columns
            .iter()
            .take(index)
            .any(|previous| previous == column)
        {
            return Err(Error::format(
                file,
                (lines + 1) as u64,
                column,
                "duplicate column",
            ));
        }
        if !expected_columns.contains(&column) {
            return Err(Error::unsupported(
                file,
                format!("unknown column {column:?}; refusing to discard it"),
            ));
        }
    }
    for column in expected_columns {
        if !columns.iter().any(|actual| actual == *column) {
            return Err(Error::format(
                file,
                (lines + 1) as u64,
                column,
                "missing column",
            ));
        }
    }
    reader
        .records()
        .map(|row| {
            let row = row.map_err(table_error)?;
            // Normalize only the file module's numeric columns, never identifiers.
            let mut normalized;
            let row = if trim_columns.is_empty() {
                &row
            } else {
                normalized = row
                    .iter()
                    .zip(columns.iter())
                    .map(|(value, name)| {
                        if trim_columns.contains(&name) {
                            value.trim()
                        } else {
                            value
                        }
                    })
                    .collect::<StringRecord>();
                normalized.set_position(row.position().cloned());
                &normalized
            };
            row.deserialize(Some(&columns)).map_err(table_error)
        })
        .collect()
}

/// Append metadata key/value lines; file modules own markers and type validation.
pub(crate) fn write_metadata<M: Serialize>(
    file: &str,
    metadata: &M,
    output: &mut String,
) -> Result<()> {
    // Reuse CSV's struct serializer for the small metadata record, then lay its
    // field/value pairs out vertically as DDES headers. This honors serde(rename)
    // and declaration order without another serializer or a handwritten field list.
    let mut header_writer = WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(Vec::new());
    header_writer
        .serialize(metadata)
        .map_err(|e| csv_error(file, 0, e))?;
    let header_bytes = header_writer
        .into_inner()
        .map_err(|e| Error::io(file, e.into_error()))?;
    let mut header_reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(header_bytes.as_slice());
    let names = header_reader
        .headers()
        .map_err(|e| csv_error(file, 0, e))?
        .clone();
    let values = header_reader
        .records()
        .next()
        .transpose()
        .map_err(|e| csv_error(file, 0, e))?
        .expect("serializing a metadata struct produces one record");
    for (key, value) in names.iter().zip(values.iter()) {
        if value.contains(['\t', '\r', '\n', '\0']) {
            return Err(Error::format(
                file,
                0,
                key,
                "metadata must fit on a single key/value line",
            ));
        }
        output.push_str(&format!("#{key}\t{value}\n"));
    }
    Ok(())
}

/// Append the table to a file-specific header, using Serde's column names/order.
pub(crate) fn write_records<'a, T: Serialize + DeserializeOwned + 'a>(
    file: &str,
    output: Vec<u8>,
    records: impl Iterator<Item = &'a T>,
) -> Result<Vec<u8>> {
    let mut writer = WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .quote_style(QuoteStyle::Always)
        .from_writer(output);
    writer
        .write_record(field_names::<T>())
        .map_err(|e| csv_error(file, 0, e))?;
    for (index, record) in records.enumerate() {
        writer
            .serialize(record)
            .map_err(|e| csv_error(file, index, e))?;
    }
    let output = writer
        .into_inner()
        .map_err(|e| Error::io(file, e.into_error()))?;
    if output.contains(&0) {
        return Err(Error::unsupported(
            file,
            "NUL bytes cannot be represented in R text tables",
        ));
    }
    Ok(output)
}

/// Ask a derived Deserialize implementation for its flat struct's wire names.
/// Used only for our record structs: no flatten, aliases, or conditional fields.
/// This also supplies headers when there are no records to serialize.
fn field_names<T: DeserializeOwned>() -> &'static [&'static str] {
    struct Fields<'a>(&'a mut &'static [&'static str]);

    impl<'de> serde::Deserializer<'de> for Fields<'_> {
        type Error = serde::de::value::Error;

        fn deserialize_any<V: serde::de::Visitor<'de>>(
            self,
            _: V,
        ) -> std::result::Result<V::Value, Self::Error> {
            Err(serde::de::Error::custom("field inspection only"))
        }

        fn deserialize_struct<V: serde::de::Visitor<'de>>(
            self,
            _: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> std::result::Result<V::Value, Self::Error> {
            *self.0 = fields;
            self.deserialize_any(visitor)
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
            map enum identifier ignored_any
        }
    }

    let mut fields = &[][..];
    // Deliberately stop after collecting field names, without constructing a row.
    let _ = T::deserialize(Fields(&mut fields));
    fields
}

/// Translate CSV locations to component line numbers, including the metadata prefix.
fn csv_error(file: &str, preceding_lines: usize, error: csv::Error) -> Error {
    let line = error
        .position()
        .map_or(0, |p| p.line() + preceding_lines as u64);
    Error::format(file, line, "table", error.to_string())
}
