use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Nights {
    #[serde(default)]
    pub nights: Vec<Night>,
}

#[derive(Debug, Deserialize)]
pub struct Night {
    pub date: NaiveDate,
    pub sleep_start_time: Option<DateTime<FixedOffset>>,
    pub sleep_end_time: Option<DateTime<FixedOffset>>,
    pub light_sleep: Option<i64>,
    pub deep_sleep: Option<i64>,
    pub rem_sleep: Option<i64>,
    pub unrecognized_sleep_stage: Option<i64>,
    pub total_interruption_duration: Option<i64>,
    pub sleep_score: Option<i64>,
    pub heart_rate_samples: Option<BTreeMap<String, f64>>,
}

#[derive(Debug, Deserialize)]
pub struct Recharges {
    #[serde(default)]
    pub recharges: Vec<Recharge>,
}

#[derive(Debug, Deserialize)]
pub struct Recharge {
    pub date: NaiveDate,
    pub heart_rate_variability_avg: Option<f64>,
    pub breathing_rate_avg: Option<f64>,
    pub ans_charge: Option<f64>,
    pub nightly_recharge_status: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Activity {
    pub start_time: String,
    pub active_duration: Option<String>,
    pub calories: Option<f64>,
    pub active_calories: Option<f64>,
    pub steps: Option<f64>,
    pub distance_from_steps: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct HeartRate {
    pub average: Option<f64>,
    pub maximum: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Exercise {
    pub id: String,
    pub start_time: NaiveDateTime,
    pub start_time_utc_offset: Option<i64>,
    pub duration: Option<String>,
    pub calories: Option<f64>,
    pub distance: Option<f64>,
    pub sport: Option<String>,
    pub detailed_sport_info: Option<String>,
    pub heart_rate: Option<HeartRate>,
}
