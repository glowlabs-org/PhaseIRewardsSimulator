use num_bigint::BigInt;
use num_traits::Signed;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serializer};
use serde_json::Value;
use std::collections::HashMap;

pub fn bigint_to_string<S>(bigint: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&bigint.to_string())
}

pub fn de_bigint<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;
    parse_value_bigint(&v).map_err(D::Error::custom)
}

fn parse_value_bigint(v: &Value) -> Result<BigInt, String> {
    match v {
        Value::String(s) => s
            .parse::<BigInt>()
            .map_err(|e| format!("invalid string for BigInt: {e}")),
        _ => Err("unsupported JSON type for BigInt, expected a string".to_string()),
    }
}

pub fn de_leftovers_map<'de, D>(deserializer: D) -> Result<HashMap<u64, BigInt>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<u64, Value> = HashMap::<u64, Value>::deserialize(deserializer)?;
    let mut out: HashMap<u64, BigInt> = HashMap::with_capacity(raw.len());
    for (k, v) in raw {
        let bi = parse_value_bigint(&v)
            .map_err(|e| DeError::custom(format!("cgp_leftovers[{k}] invalid: {e}")))?;
        if bi.is_negative() {
            return Err(DeError::custom(format!(
                "cgp_leftovers[{k}] cannot be negative"
            )));
        }
        out.insert(k, bi);
    }
    Ok(out)
}

pub fn de_optional_gctl_map<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<u64, BigInt>>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;
    if v.is_null() {
        return Ok(None);
    }

    let raw: HashMap<u64, Value> = serde_json::from_value(v)
        .map_err(|e| DeError::custom(format!("gctlDistribution must be a map or null: {e}")))?;

    let mut out: HashMap<u64, BigInt> = HashMap::with_capacity(raw.len());
    for (k, v) in raw {
        let bi = parse_value_bigint(&v)
            .map_err(|e| DeError::custom(format!("gctlDistribution[{k}] invalid: {e}")))?;
        if bi.is_negative() {
            return Err(DeError::custom(format!(
                "gctlDistribution[{k}] cannot be negative"
            )));
        }
        out.insert(k, bi);
    }
    Ok(Some(out))
}
