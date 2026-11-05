//! Numeric validation and TSV text-cell codecs.

use serde::{Deserialize, Deserializer, Serializer, de::Error as _, ser::Error as _};

fn check_float<const NONNEGATIVE: bool>(value: f64) -> Result<f64, &'static str> {
    if !value.is_finite() {
        Err("expected a finite number")
    } else if NONNEGATIVE && value < 0.0 {
        Err("expected a nonnegative number")
    } else {
        Ok(value)
    }
}

fn parse_float<const NONNEGATIVE: bool>(raw: &str) -> Result<f64, &'static str> {
    let value = raw.trim().parse().map_err(|_| "expected a number")?;
    check_float::<NONNEGATIVE>(value)
}

pub(crate) mod optional_count {
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
        let raw = String::deserialize(d)?;
        let token = raw.trim();
        if token == "NA" {
            return Ok(None);
        }
        token.parse().map(Some).map_err(D::Error::custom)
    }

    pub fn serialize<S: Serializer>(value: &Option<u64>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&value.map_or_else(|| "NA".into(), |n| n.to_string()))
    }
}

fn read_optional_float<'de, const NONNEGATIVE: bool, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<f64>, D::Error> {
    let raw = String::deserialize(d)?;
    if raw.trim() == "NA" {
        Ok(None)
    } else {
        parse_float::<NONNEGATIVE>(&raw)
            .map(Some)
            .map_err(D::Error::custom)
    }
}

fn write_optional_float<const NONNEGATIVE: bool, S: Serializer>(
    value: &Option<f64>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match value {
        None => s.serialize_str("NA"),
        Some(value) => {
            check_float::<NONNEGATIVE>(*value).map_err(S::Error::custom)?;
            s.serialize_str(&value.to_string())
        }
    }
}

pub(crate) mod optional_threshold {
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
        read_optional_float::<false, D>(d)
    }

    pub fn serialize<S: Serializer>(value: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
        write_optional_float::<false, S>(value, s)
    }
}

pub(crate) mod optional_concentration {
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
        read_optional_float::<true, D>(d)
    }

    pub fn serialize<S: Serializer>(value: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
        write_optional_float::<true, S>(value, s)
    }
}

/// Native numeric Serde, retaining the endpoint finiteness rule outside TSV too.
pub(crate) mod finite_intensity {
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
        check_float::<false>(f64::deserialize(d)?).map_err(D::Error::custom)
    }

    pub fn serialize<S: Serializer>(value: &f64, s: S) -> Result<S::Ok, S::Error> {
        check_float::<false>(*value).map_err(S::Error::custom)?;
        s.serialize_f64(*value)
    }
}

pub(crate) mod probe_concentrations {
    use super::*;

    fn validate(raw: &str) -> Result<(), &'static str> {
        for token in raw.split(';') {
            if token.trim() != "NA" {
                parse_float::<true>(token)?;
            }
        }
        Ok(())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        let raw = String::deserialize(d)?;
        validate(&raw).map_err(D::Error::custom)?;
        Ok(raw)
    }

    pub fn serialize<S: Serializer>(value: &str, s: S) -> Result<S::Ok, S::Error> {
        validate(value).map_err(S::Error::custom)?;
        s.serialize_str(value)
    }
}
