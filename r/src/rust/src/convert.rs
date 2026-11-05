//! Checked conversion of named R lists and data-frame columns.

use extendr_api::prelude::*;

use crate::Result;

pub(crate) fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}

pub(crate) fn fields(value: Robj, expected: &[&str]) -> Result<List> {
    let list = value.as_list().ok_or("expected a named list")?;
    let names: Vec<_> = list.names().ok_or("expected field names")?.collect();
    if names.len() != expected.len()
        || expected
            .iter()
            .any(|key| names.iter().filter(|n| *n == key).count() != 1)
    {
        return Err(format!(
            "expected exactly these fields: {}",
            expected.join(", ")
        ));
    }
    Ok(list)
}

pub(crate) fn get(list: &List, name: &str) -> Result<Robj> {
    list.iter()
        .find(|(key, _)| *key == name)
        .map(|(_, v)| v)
        .ok_or_else(|| format!("missing field {name}"))
}

pub(crate) fn strings(list: &List, name: &str) -> Result<Vec<String>> {
    let value = get(list, name)?;
    if !value.is_string() || value.is_object() || value.dim().is_some() {
        return Err(format!("{name}: expected an unclassed character vector"));
    }
    let values = value
        .as_str_vector()
        .ok_or_else(|| format!("{name}: expected character values"))?;
    values
        .into_iter()
        .map(|v| {
            if v.is_na() {
                Err(format!("{name}: NA is not allowed"))
            } else {
                Ok(v.to_owned())
            }
        })
        .collect()
}

pub(crate) fn text(list: &List, name: &str) -> Result<String> {
    let mut values = strings(list, name)?;
    if values.len() != 1 {
        return Err(format!("{name}: expected one string"));
    }
    Ok(values.remove(0))
}

pub(crate) fn counts(list: &List, name: &str) -> Result<Vec<Option<u64>>> {
    let value = get(list, name)?;
    if !value.is_string() || value.is_object() || value.dim().is_some() {
        return Err(format!(
            "{name}: expected an unclassed decimal character vector"
        ));
    }
    let values = value
        .as_str_vector()
        .ok_or_else(|| format!("{name}: expected decimal character values or NA_character_"))?;
    values
        .into_iter()
        .map(|v| {
            if v.is_na() {
                return Ok(None);
            }
            if v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
                return Err(format!(
                    "{name}: expected unsigned decimal digits or NA_character_"
                ));
            }
            v.parse::<u64>()
                .map(Some)
                .map_err(|_| format!("{name}: count exceeds u64 range"))
        })
        .collect()
}

pub(crate) fn numbers(list: &List, name: &str) -> Result<Vec<Option<f64>>> {
    let value = get(list, name)?;
    if value.is_object() || value.dim().is_some() {
        return Err(format!("{name}: expected an unclassed numeric vector"));
    }
    if let Some(values) = value.as_real_slice() {
        Ok(values
            .iter()
            .map(|&v| if v.is_na() { None } else { Some(v) })
            .collect())
    } else if let Some(values) = value.as_integer_slice() {
        Ok(values
            .iter()
            .map(|&v| if v.is_na() { None } else { Some(f64::from(v)) })
            .collect())
    } else {
        Err(format!("{name}: expected numeric values"))
    }
}

pub(crate) fn records(file: &List, expected: &[&str]) -> Result<(List, usize)> {
    let value = get(file, "records")?;
    if !value.inherits("data.frame") {
        return Err("records: expected a data.frame".into());
    }
    let list = fields(value, expected)?;
    let len = list.elt(0).map_err(error)?.len();
    if list.values().any(|v| v.len() != len || v.dim().is_some()) {
        return Err("records: columns must be vectors of equal length".into());
    }
    Ok((list, len))
}

pub(crate) fn dataframe(mut columns: List, len: usize) -> Result<List> {
    let len = i32::try_from(len).map_err(|_| "too many rows for an R data frame")?;
    let rows = if len == 0 {
        Vec::new()
    } else {
        vec![i32::na(), -len]
    };
    columns.set_attrib("row.names", rows).map_err(error)?;
    columns.set_class(["data.frame"]).map_err(error)?;
    Ok(columns)
}

pub(crate) fn count_values(values: impl ExactSizeIterator<Item = Option<u64>>) -> Strings {
    Strings::from_values(values.map(|v| Rstr::from(v.map(|n| n.to_string()))))
}

pub(crate) fn number_values(values: impl ExactSizeIterator<Item = Option<f64>>) -> Doubles {
    Doubles::from_values(values.map(Rfloat::from))
}
