use crate::Coupon;

use serde::{de, Deserialize};

use std::collections::HashMap;

pub fn yn<'de, D>(d: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    match <&str>::deserialize(d)? {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => Err(de::Error::custom("Invalid value")),
    }
}

pub fn money<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = <&str>::deserialize(d)?;
    s.chars()
        .filter(|&c| c != ',')
        .collect::<String>()
        .parse()
        .map_err(|_| de::Error::custom(format!("Can not parse f64 from {}", s)))
}

pub fn coupon<'de, D>(d: D) -> Result<HashMap<String, Coupon>, D::Error>
where
    D: de::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        Empty(Vec<u8>),
        Some(HashMap<String, Coupon>),
    }

    match Value::deserialize(d)? {
        Value::Empty(v) => {
            if v.len() == 0 {
                Ok(HashMap::new())
            } else {
                Err(de::Error::custom(
                    "Could not parse coupons field, found non-empty list",
                ))
            }
        }
        Value::Some(m) => Ok(m),
    }
}

pub fn strnum<'de, D>(d: D) -> Result<u64, D::Error>
where
    D: de::Deserializer<'de>,
{
    <&str>::deserialize(d)?
        .parse()
        .map_err(|e| de::Error::custom(format!("Parse Error: {:?}", e)))
}

pub fn opt_strnum<'de, D>(d: D) -> Result<Option<u64>, D::Error>
where
    D: de::Deserializer<'de>,
{
    <Option<String>>::deserialize(d)?
        .map(|s| {
            s.parse()
                .map_err(|e| de::Error::custom(format!("Parse Error: {:?}", e)))
        })
        .transpose()
}
