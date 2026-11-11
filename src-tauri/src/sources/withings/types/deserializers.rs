use serde::Deserialize;

use super::heart::HeartHr;

pub fn deserialize_more<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum MoreValue {
        Int(i64),
        Bool(bool),
    }

    let opt = Option::<MoreValue>::deserialize(deserializer)?;
    Ok(opt.map(|v| match v {
        MoreValue::Int(i) => i,
        MoreValue::Bool(b) => {
            if b {
                1
            } else {
                0
            }
        }
    }))
}

pub fn deserialize_lenient_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(val.and_then(|v| match v {
        serde_json::Value::Number(n) => n.as_i64(),
        _ => None,
    }))
}

pub fn deserialize_lenient_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(val.and_then(|v| match v {
        serde_json::Value::Number(n) => n.as_f64(),
        _ => None,
    }))
}

pub fn deserialize_heart_rate<'de, D>(deserializer: D) -> Result<Option<HeartHr>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(val.and_then(|v| match v {
        serde_json::Value::Object(map) => {
            let value = map.get("value").and_then(|v| v.as_i64());
            let qrs = map.get("qrs_duration").and_then(|v| v.as_i64());
            Some(HeartHr {
                value,
                qrs_duration: qrs,
            })
        }
        serde_json::Value::Number(n) => Some(HeartHr {
            value: n.as_i64(),
            qrs_duration: None,
        }),
        _ => None,
    }))
}
