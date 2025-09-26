use num_bigint::BigInt;
use num_traits::Signed;
use serde::de::{Error as DeError, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

pub fn de_bigint<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: Deserializer<'de>,
{
    struct BigIntVisitor;

    impl<'de> Visitor<'de> for BigIntVisitor {
        type Value = BigInt;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "a JSON integer or a string representing an integer")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(BigInt::from(v))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(BigInt::from(v))
        }

        fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(BigInt::from(v))
        }

        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(BigInt::from(v))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            parse_str_bigint(v).map_err(|_| DeError::invalid_value(Unexpected::Str(v), &self))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            self.visit_str(&v)
        }
    }

    deserializer.deserialize_any(BigIntVisitor)
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
        let bi = match v {
            Value::Number(n) => {
                if let Some(u) = n.as_u64() {
                    BigInt::from(u)
                } else if let Some(i) = n.as_i64() {
                    if i < 0 {
                        return Err(DeError::custom(format!(
                            "cgp_leftovers[{k}] cannot be negative"
                        )));
                    }
                    BigInt::from(i)
                } else {
                    return Err(DeError::custom(format!(
                        "cgp_leftovers[{k}] must be an integer"
                    )));
                }
            }
            Value::String(s) => parse_str_bigint(&s).map_err(|_| {
                DeError::custom(format!(
                    "cgp_leftovers[{k}] string is not a valid integer: {s}"
                ))
            })?,
            other => {
                return Err(DeError::custom(format!(
                    "cgp_leftovers[{k}] must be integer or string, got {other:?}"
                )))
            }
        };
        if bi.is_negative() {
            return Err(DeError::custom(format!(
                "cgp_leftovers[{k}] cannot be negative"
            )));
        }
        out.insert(k, bi);
    }
    Ok(out)
}
