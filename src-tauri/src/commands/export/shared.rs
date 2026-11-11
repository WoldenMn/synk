use crate::error::AppError;
use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::Manager;

#[derive(sqlx::FromRow)]
pub(super) struct BmRow {
    pub(super) measured_at: String,
    pub(super) weight: Option<f64>,
    pub(super) fat_ratio: Option<f64>,
    pub(super) fat_mass: Option<f64>,
    pub(super) fat_free_mass: Option<f64>,
    pub(super) muscle_mass: Option<f64>,
    pub(super) bone_mass: Option<f64>,
    pub(super) hydration: Option<f64>,
    pub(super) bmi: Option<f64>,
}

#[derive(sqlx::FromRow)]
pub(super) struct BpRow {
    pub(super) measured_at: String,
    pub(super) systolic: i64,
    pub(super) diastolic: i64,
    pub(super) heart_rate: Option<i64>,
}

#[derive(sqlx::FromRow)]
pub(super) struct SleepRow {
    pub(super) date: String,
    pub(super) start_at: Option<String>,
    pub(super) end_at: Option<String>,
    pub(super) duration_total: Option<i64>,
    pub(super) sleep_score: Option<i64>,
}

#[derive(sqlx::FromRow)]
pub(super) struct ActivityRow {
    pub(super) date: String,
    pub(super) steps: Option<i64>,
    pub(super) distance: Option<f64>,
    pub(super) calories_active: Option<f64>,
}

#[derive(sqlx::FromRow)]
pub(super) struct MeasRow {
    pub(super) measured_at: String,
    pub(super) label: String,
    pub(super) unit: String,
    pub(super) value: f64,
}

#[derive(sqlx::FromRow)]
pub(super) struct WorkoutRow {
    pub(super) date: String,
    pub(super) category_label: Option<String>,
    pub(super) duration: Option<i64>,
    pub(super) distance: Option<f64>,
    pub(super) calories: Option<f64>,
    pub(super) hr_average: Option<i64>,
}

#[derive(sqlx::FromRow)]
pub(super) struct HeartRow {
    pub(super) timestamp: String,
    pub(super) ecg_status: Option<i64>,
    pub(super) heart_rate: Option<i64>,
}

pub(super) fn resolve_output_dir(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .download_dir()
        .or_else(|_| {
            app.path().app_data_dir().map_err(|e| {
                AppError::Internal(format!(
                    "Impossible de resoudre le dossier de sortie: {}",
                    e
                ))
            })
        })
        .map_err(|e| {
            AppError::Internal(format!(
                "Impossible de resoudre le dossier de sortie: {}",
                e
            ))
        })
}

pub(super) async fn fetch_body_measures(
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<Vec<BmRow>, AppError> {
    sqlx::query_as::<_, BmRow>(
        "SELECT measured_at, weight, fat_ratio, fat_mass, fat_free_mass, muscle_mass, bone_mass, hydration, bmi \
         FROM body_measures \
         WHERE measured_at >= ?1 AND measured_at < ?2 \
         ORDER BY measured_at ASC",
    )
    .bind(date_from)
    .bind(date_before)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub(super) async fn fetch_blood_pressure(
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<Vec<BpRow>, AppError> {
    sqlx::query_as::<_, BpRow>(
        "SELECT measured_at, systolic, diastolic, heart_rate \
         FROM blood_pressure \
         WHERE measured_at >= ?1 AND measured_at < ?2 \
         ORDER BY measured_at ASC",
    )
    .bind(date_from)
    .bind(date_before)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub(super) async fn fetch_sleep(
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<Vec<SleepRow>, AppError> {
    sqlx::query_as::<_, SleepRow>(
        "SELECT date, start_at, end_at, duration_total, sleep_score \
         FROM sleep_records \
         WHERE date >= ?1 AND date < ?2 \
         ORDER BY date ASC",
    )
    .bind(date_from)
    .bind(date_before)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub(super) async fn fetch_activity(
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<Vec<ActivityRow>, AppError> {
    sqlx::query_as::<_, ActivityRow>(
        "SELECT date, steps, distance, calories_active \
         FROM activity \
         WHERE date >= ?1 AND date < ?2 \
         ORDER BY date ASC",
    )
    .bind(date_from)
    .bind(date_before)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub(super) async fn fetch_measurements(
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<Vec<MeasRow>, AppError> {
    sqlx::query_as::<_, MeasRow>(
        "SELECT mm.measured_at, mt.label, mt.unit, mm.value \
         FROM manual_measurements mm \
         JOIN measurement_types mt ON mt.id = mm.measurement_type_id \
         WHERE mm.measured_at >= ?1 AND mm.measured_at < ?2 \
         ORDER BY mm.measured_at ASC",
    )
    .bind(date_from)
    .bind(date_before)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub(super) async fn fetch_workouts(
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<Vec<WorkoutRow>, AppError> {
    sqlx::query_as::<_, WorkoutRow>(
        "SELECT date, category_label, duration, distance, calories, hr_average \
         FROM workouts \
         WHERE date >= ?1 AND date < ?2 \
         ORDER BY date ASC",
    )
    .bind(date_from)
    .bind(date_before)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub(super) async fn fetch_heart(
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<Vec<HeartRow>, AppError> {
    sqlx::query_as::<_, HeartRow>(
        "SELECT timestamp, ecg_status, heart_rate \
         FROM heart_records \
         WHERE timestamp >= ?1 AND timestamp < ?2 \
         ORDER BY timestamp ASC",
    )
    .bind(date_from)
    .bind(date_before)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

#[cfg(test)]
mod duplication_guard {
    #[test]
    fn csv_and_pdf_read_through_the_shared_queries() {
        let sources = [
            ("csv/mod.rs", include_str!("csv/mod.rs")),
            ("csv/body.rs", include_str!("csv/body.rs")),
            ("csv/daily.rs", include_str!("csv/daily.rs")),
            ("pdf/fetch.rs", include_str!("pdf/fetch.rs")),
        ];
        for (name, source) in sources {
            let selects = source.matches("SELECT").count();
            assert_eq!(
                selects, 0,
                "{name} contient encore {selects} requete(s) SELECT"
            );
        }
    }
}
