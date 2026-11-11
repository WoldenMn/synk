use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Page<T> {
    #[serde(default = "Vec::new")]
    pub records: Vec<T>,
    #[serde(default)]
    pub next_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StageSummary {
    pub total_in_bed_time_milli: Option<f64>,
    pub total_awake_time_milli: Option<f64>,
    pub total_light_sleep_time_milli: Option<f64>,
    pub total_slow_wave_sleep_time_milli: Option<f64>,
    pub total_rem_sleep_time_milli: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct SleepScore {
    pub stage_summary: Option<StageSummary>,
    pub respiratory_rate: Option<f64>,
    pub sleep_performance_percentage: Option<f64>,
    pub sleep_efficiency_percentage: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Sleep {
    pub id: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    #[serde(default)]
    pub timezone_offset: Option<String>,
    #[serde(default)]
    pub nap: bool,
    #[serde(default)]
    pub score_state: Option<String>,
    #[serde(default)]
    pub score: Option<SleepScore>,
}

#[derive(Debug, Deserialize)]
pub struct RecoveryScore {
    pub recovery_score: Option<f64>,
    pub resting_heart_rate: Option<f64>,
    pub hrv_rmssd_milli: Option<f64>,
    pub spo2_percentage: Option<f64>,
    pub skin_temp_celsius: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Recovery {
    pub sleep_id: String,
    #[serde(default)]
    pub score_state: Option<String>,
    #[serde(default)]
    pub score: Option<RecoveryScore>,
}

#[derive(Debug, Deserialize)]
pub struct CycleScore {
    pub strain: Option<f64>,
    pub kilojoule: Option<f64>,
    pub average_heart_rate: Option<f64>,
    pub max_heart_rate: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Cycle {
    pub id: i64,
    pub start: DateTime<Utc>,
    #[serde(default)]
    pub timezone_offset: Option<String>,
    #[serde(default)]
    pub score_state: Option<String>,
    #[serde(default)]
    pub score: Option<CycleScore>,
}

#[derive(Debug, Deserialize)]
pub struct WorkoutScore {
    pub average_heart_rate: Option<f64>,
    pub max_heart_rate: Option<f64>,
    pub kilojoule: Option<f64>,
    pub distance_meter: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Workout {
    pub id: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    #[serde(default)]
    pub timezone_offset: Option<String>,
    #[serde(default)]
    pub sport_name: Option<String>,
    #[serde(default)]
    pub score_state: Option<String>,
    #[serde(default)]
    pub score: Option<WorkoutScore>,
}
