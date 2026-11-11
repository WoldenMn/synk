use serde::Deserialize;

use super::deserializers::{deserialize_lenient_f64, deserialize_lenient_i64, deserialize_more};

#[derive(Debug, Deserialize)]
pub struct SleepBody {
    #[serde(default)]
    pub series: Vec<SleepEntry>,
    #[serde(default, deserialize_with = "deserialize_more")]
    pub more: Option<i64>,
    #[allow(dead_code)]
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SleepEntry {
    pub date: String,
    pub startdate: Option<i64>,
    pub enddate: Option<i64>,
    #[serde(default)]
    pub data: SleepData,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct SleepData {
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub deepsleepduration: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub lightsleepduration: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub remsleepduration: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub wakeupduration: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub durationtosleep: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub durationtowakeup: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub sleep_score: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub hr_average: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub hr_min: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub hr_max: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_f64")]
    pub rr_average: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub snoring: Option<i64>,
    #[serde(
        default,
        alias = "snoring_episode_count",
        deserialize_with = "deserialize_lenient_i64"
    )]
    pub snoringepisodecount: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub total_sleep_time: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub night_events: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_f64")]
    pub sleep_efficiency: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub nb_rem_episodes: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub out_of_bed_count: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub waso: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub total_timeinbed: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_lenient_f64")]
    pub apnea_hypopnea_index: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_lenient_f64")]
    pub breathing_disturbances_intensity: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    pub asleepduration: Option<i64>,
}
