use serde::Deserialize;

use super::deserializers::deserialize_more;

#[derive(Debug, Deserialize)]
pub struct WorkoutBody {
    #[serde(default)]
    pub series: Vec<WorkoutEntry>,
    #[serde(default, deserialize_with = "deserialize_more")]
    pub more: Option<i64>,
    #[allow(dead_code)]
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct WorkoutEntry {
    pub id: Option<i64>,
    pub category: Option<i64>,
    pub startdate: Option<i64>,
    pub enddate: Option<i64>,
    pub date: Option<String>,
    #[serde(default)]
    pub data: WorkoutData,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct WorkoutData {
    pub steps: Option<i64>,
    pub distance: Option<f64>,
    pub elevation: Option<f64>,
    pub calories: Option<f64>,
    pub hr_average: Option<i64>,
    pub hr_min: Option<i64>,
    pub hr_max: Option<i64>,
    pub hr_zone_0: Option<i64>,
    pub hr_zone_1: Option<i64>,
    pub hr_zone_2: Option<i64>,
    pub hr_zone_3: Option<i64>,
    pub spo2_average: Option<f64>,
    pub intensity: Option<i64>,
    pub pause_duration: Option<i64>,
    #[serde(rename = "effortScore")]
    pub effort_score: Option<f64>,
    pub pool_laps: Option<i64>,
    pub pool_length: Option<i64>,
    #[allow(dead_code)]
    pub manual_distance: Option<f64>,
    #[allow(dead_code)]
    pub manual_calories: Option<f64>,
}
