use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Page<T> {
    #[serde(default = "Vec::new")]
    pub data: Vec<T>,
    #[serde(default)]
    pub next_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DailySleep {
    pub id: String,
    pub day: NaiveDate,
    pub score: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct SleepPeriod {
    pub id: String,
    pub day: NaiveDate,
    pub bedtime_start: DateTime<FixedOffset>,
    pub bedtime_end: DateTime<FixedOffset>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub total_sleep_duration: Option<f64>,
    pub deep_sleep_duration: Option<f64>,
    pub light_sleep_duration: Option<f64>,
    pub rem_sleep_duration: Option<f64>,
    pub awake_time: Option<f64>,
    pub latency: Option<f64>,
    pub time_in_bed: Option<f64>,
    pub efficiency: Option<f64>,
    pub average_heart_rate: Option<f64>,
    pub lowest_heart_rate: Option<f64>,
    pub average_hrv: Option<f64>,
    pub average_breath: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct DailyActivity {
    pub id: String,
    pub day: NaiveDate,
    pub steps: Option<f64>,
    pub active_calories: Option<f64>,
    pub total_calories: Option<f64>,
    pub equivalent_walking_distance: Option<f64>,
    pub low_activity_time: Option<f64>,
    pub medium_activity_time: Option<f64>,
    pub high_activity_time: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct DailyReadiness {
    pub id: String,
    pub day: NaiveDate,
    pub score: Option<f64>,
    pub temperature_deviation: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Spo2Average {
    pub average: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct DailySpo2 {
    pub id: String,
    pub day: NaiveDate,
    pub spo2_percentage: Option<Spo2Average>,
}

#[derive(Debug, Deserialize)]
pub struct Workout {
    pub id: String,
    pub activity: String,
    pub day: NaiveDate,
    pub start_datetime: DateTime<FixedOffset>,
    pub end_datetime: DateTime<FixedOffset>,
    pub calories: Option<f64>,
    pub distance: Option<f64>,
}
