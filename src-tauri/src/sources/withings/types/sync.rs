use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub body_measures_count: usize,
    pub blood_pressure_count: usize,
    pub sleep_count: usize,
    pub activity_count: usize,
    pub vitals_count: usize,
    pub workouts_count: usize,
    pub heart_count: usize,
    pub devices_count: usize,
    pub errors: Vec<String>,
}
