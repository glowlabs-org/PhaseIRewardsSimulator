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
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Ok(BigInt::from(u))
            } else if let Some(i) = n.as_i64() {
                Ok(BigInt::from(i))
            } else if let Some(i) = n.as_i128() {
                Ok(BigInt::from(i))
            } else {
                Err("invalid numeric value for BigInt".to_string())
            }
        }
        Value::String(s) => {
            parse_str_bigint(s).map_err(|_| "invalid string for BigInt".to_string())
        }
        Value::Array(arr) => parse_array_bigint(arr),
        _ => Err("unsupported JSON type for BigInt".to_string()),
    }
}

fn parse_array_bigint(arr: &[Value]) -> Result<BigInt, String> {
    if arr.len() != 2 {
        return Err("BigInt array form must be [sign, [limbs]]".to_string());
    }
    let sign_val = &arr[0];
    let limbs_val = &arr[1];

    let sign = match sign_val {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i
            } else if let Some(i) = n.as_i128() {
                i as i64
            } else {
                return Err("BigInt array form sign must be integer".to_string());
            }
        }
        Value::String(s) => s
            .parse::<i64>()
            .map_err(|_| "BigInt array form sign must be integer".to_string())?,
        _ => return Err("BigInt array form sign must be integer".to_string()),
    };

    let limbs = match limbs_val {
        Value::Array(v) => v,
        _ => return Err("BigInt array form limbs must be array".to_string()),
    };

    let base = BigInt::from(1u128 << 32);
    let mut acc = BigInt::from(0u32);
    for limb in limbs.iter().rev() {
        acc *= &base;
        let lv = match limb {
            Value::Number(n) => {
                if let Some(u) = n.as_u64() {
                    BigInt::from(u)
                } else if let Some(i) = n.as_i64() {
                    if i < 0 {
                        return Err("BigInt limb cannot be negative".to_string());
                    }
                    BigInt::from(i as u64)
                } else {
                    return Err("invalid limb in BigInt array form".to_string());
                }
            }
            Value::String(s) => {
                let ds = s.trim();
                if ds.starts_with('-') {
                    return Err("BigInt limb cannot be negative".to_string());
                }
                ds.parse::<BigInt>()
                    .map_err(|_| "invalid limb string".to_string())?
            }
            _ => return Err("invalid limb type".to_string()),
        };
        acc += lv;
    }

    if sign < 0 {
        Ok(-acc)
    } else if sign == 0 {
        Ok(BigInt::from(0u32))
    } else {
        Ok(acc)
    }
}

fn parse_str_bigint(s: &str) -> Result<BigInt, ()> {
    if s.trim().is_empty() {
        return Err(());
    }
    if s.contains('.') {
        return Err(());
    }
    s.parse::<BigInt>().map_err(|_| ())
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

pub fn de_region_id_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;
    match v {
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                let s = match u {
                    1 => "cgp".to_string(),
                    2 => "utah".to_string(),
                    x => x.to_string(),
                };
                Ok(s)
            } else {
                Err(D::Error::custom("regionId number must be unsigned integer"))
            }
        }
        Value::String(s) => Ok(s),
        other => Err(D::Error::custom(format!(
            "invalid regionId type: expected string or number, got {other:?}"
        ))),
    }
}
