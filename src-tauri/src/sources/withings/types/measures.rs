use serde::Deserialize;

use super::deserializers::deserialize_more;

#[derive(Debug, Deserialize)]
pub struct MeasureBody {
    #[serde(default)]
    pub measuregrps: Vec<MeasureGroup>,
    #[serde(default, deserialize_with = "deserialize_more")]
    pub more: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MeasureGroup {
    pub grpid: i64,
    pub date: i64,
    #[serde(default)]
    pub measures: Vec<Measure>,
}

#[derive(Debug, Deserialize)]
pub struct Measure {
    pub value: i64,
    #[serde(rename = "type")]
    pub measure_type: i64,
    pub unit: i64,
}
