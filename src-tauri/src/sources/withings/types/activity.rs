use serde::Deserialize;

use super::deserializers::deserialize_more;

#[derive(Debug, Deserialize)]
pub struct ActivityBody {
    #[serde(default)]
    pub activities: Vec<ActivityEntry>,
    #[serde(default, deserialize_with = "deserialize_more")]
    pub more: Option<i64>,
    #[allow(dead_code)]
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct ActivityEntry {
    pub date: String,
    pub steps: Option<i64>,
    pub distance: Option<f64>,
    pub elevation: Option<f64>,
    pub soft: Option<i64>,
    pub moderate: Option<i64>,
    pub intense: Option<i64>,
    pub active: Option<i64>,
    pub calories: Option<f64>,
    pub totalcalories: Option<f64>,
    pub hr_average: Option<i64>,
    pub hr_min: Option<i64>,
    pub hr_max: Option<i64>,
    pub hr_zone_0: Option<i64>,
    pub hr_zone_1: Option<i64>,
    pub hr_zone_2: Option<i64>,
    pub hr_zone_3: Option<i64>,
}
