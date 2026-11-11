use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Flex {
    Int(i64),
    Float(f64),
    Text(String),
}

impl Flex {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Flex::Int(i) => Some(*i as f64),
            Flex::Float(f) => Some(*f),
            Flex::Text(s) => s.trim().parse().ok(),
        }
    }
}

pub fn number(value: &Option<Flex>) -> Option<f64> {
    value
        .as_ref()
        .and_then(Flex::as_f64)
        .filter(|v| v.is_finite())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<P> {
    #[serde(default = "Vec::new")]
    pub data_points: Vec<P>,
    #[serde(default)]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Day {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Day {
    pub fn date(&self) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(self.year, self.month, self.day)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interval {
    pub start_time: DateTime<Utc>,
    #[serde(default)]
    pub start_utc_offset: Option<String>,
    pub end_time: DateTime<Utc>,
    #[serde(default)]
    pub end_utc_offset: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepPoint {
    #[serde(default)]
    pub name: Option<String>,
    pub sleep: Sleep,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sleep {
    pub interval: Interval,
    #[serde(default)]
    pub metadata: Option<SleepMetadata>,
    #[serde(default)]
    pub summary: Option<SleepSummary>,
}

#[derive(Debug, Deserialize)]
pub struct SleepMetadata {
    #[serde(default, alias = "mainSleep")]
    pub main: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepSummary {
    #[serde(default)]
    pub minutes_in_sleep_period: Option<Flex>,
    #[serde(default)]
    pub minutes_to_fall_asleep: Option<Flex>,
    #[serde(default)]
    pub minutes_asleep: Option<Flex>,
    #[serde(default)]
    pub minutes_awake: Option<Flex>,
    #[serde(default)]
    pub stages_summary: Vec<StageSummary>,
}

#[derive(Debug, Deserialize)]
pub struct StageSummary {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub minutes: Option<Flex>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExercisePoint {
    #[serde(default)]
    pub name: Option<String>,
    pub exercise: Exercise,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    pub interval: Interval,
    #[serde(default)]
    pub exercise_type: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub active_duration: Option<String>,
    #[serde(default)]
    pub metrics_summary: Option<ExerciseMetrics>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseMetrics {
    #[serde(default)]
    pub calories_kcal: Option<Flex>,
    #[serde(default, alias = "distanceMillimiters")]
    pub distance_millimeters: Option<Flex>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestingHeartRatePoint {
    pub daily_resting_heart_rate: RestingHeartRate,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestingHeartRate {
    pub date: Day,
    #[serde(default)]
    pub beats_per_minute: Option<Flex>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HrvPoint {
    pub daily_heart_rate_variability: Hrv,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hrv {
    pub date: Day,
    #[serde(default)]
    pub average_heart_rate_variability_milliseconds: Option<Flex>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spo2Point {
    pub daily_oxygen_saturation: Spo2,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spo2 {
    pub date: Day,
    #[serde(default)]
    pub average_percentage: Option<Flex>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemperaturePoint {
    pub daily_sleep_temperature_derivations: Temperature,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Temperature {
    pub date: Day,
    #[serde(default)]
    pub nightly_temperature_celsius: Option<Flex>,
    #[serde(default)]
    pub baseline_temperature_celsius: Option<Flex>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RespiratoryPoint {
    pub daily_respiratory_rate: Respiratory,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Respiratory {
    pub date: Day,
    #[serde(default)]
    pub breaths_per_minute: Option<Flex>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rollups {
    #[serde(default, alias = "dailyRollupDataPoints")]
    pub rollup_data_points: Vec<Rollup>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rollup {
    pub civil_start_time: CivilTime,
    #[serde(default)]
    pub steps: Option<CountSum>,
    #[serde(default)]
    pub distance: Option<MillimetersSum>,
    #[serde(default)]
    pub active_energy_burned: Option<KcalSum>,
    #[serde(default)]
    pub total_calories: Option<KcalSum>,
}

#[derive(Debug, Deserialize)]
pub struct CivilTime {
    pub date: Day,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountSum {
    #[serde(default)]
    pub count_sum: Option<Flex>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MillimetersSum {
    #[serde(default)]
    pub millimeters_sum: Option<Flex>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KcalSum {
    #[serde(default)]
    pub kcal_sum: Option<Flex>,
}
