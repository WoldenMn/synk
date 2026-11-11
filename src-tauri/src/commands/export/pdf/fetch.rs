use crate::error::AppError;
use sqlx::SqlitePool;

use crate::commands::export::shared::{
    fetch_activity, fetch_blood_pressure, fetch_body_measures, fetch_heart, fetch_measurements,
    fetch_sleep, fetch_workouts, ActivityRow, BmRow, BpRow, HeartRow, MeasRow, SleepRow,
    WorkoutRow,
};

pub(super) struct PdfData {
    pub(super) bm_rows: Vec<BmRow>,
    pub(super) bp_rows: Vec<BpRow>,
    pub(super) sleep_rows: Vec<SleepRow>,
    pub(super) activity_rows: Vec<ActivityRow>,
    pub(super) meas_rows: Vec<MeasRow>,
    pub(super) workout_rows: Vec<WorkoutRow>,
    pub(super) heart_rows: Vec<HeartRow>,
}

impl PdfData {
    pub(super) fn row_count(&self) -> u64 {
        (self.bm_rows.len()
            + self.bp_rows.len()
            + self.sleep_rows.len()
            + self.activity_rows.len()
            + self.meas_rows.len()
            + self.workout_rows.len()
            + self.heart_rows.len()) as u64
    }
}

pub(super) async fn fetch_pdf_data(
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
    categories: &[String],
) -> Result<PdfData, AppError> {
    let mut data = PdfData {
        bm_rows: vec![],
        bp_rows: vec![],
        sleep_rows: vec![],
        activity_rows: vec![],
        meas_rows: vec![],
        workout_rows: vec![],
        heart_rows: vec![],
    };

    for category in categories {
        match category.as_str() {
            "body_measures" => {
                data.bm_rows = fetch_body_measures(pool, date_from, date_before).await?
            }
            "blood_pressure" => {
                data.bp_rows = fetch_blood_pressure(pool, date_from, date_before).await?
            }
            "sleep" => data.sleep_rows = fetch_sleep(pool, date_from, date_before).await?,
            "activity" => data.activity_rows = fetch_activity(pool, date_from, date_before).await?,
            "measurements" => {
                data.meas_rows = fetch_measurements(pool, date_from, date_before).await?
            }
            "workouts" => data.workout_rows = fetch_workouts(pool, date_from, date_before).await?,
            "heart" => data.heart_rows = fetch_heart(pool, date_from, date_before).await?,
            _ => {}
        }
    }

    Ok(data)
}
