use crate::error::AppError;
use sqlx::SqlitePool;

use super::client;
use super::types::*;
use crate::sources::SourceId;

pub mod activity;
pub use activity::sync_activity;
mod devices;
pub use devices::sync_devices;
mod heart;
pub use heart::sync_heart;
mod helpers;
pub(super) use helpers::{compute_ymd_range, measured_at_iso, paginate_batch};
mod measures;
pub use measures::sync_body_measures;
mod metrics;
pub mod sleep;
pub use sleep::sync_sleep;
mod sync_state;
pub use sync_state::{get_last_sync, update_last_sync};
mod time;
pub(super) use time::{opt_ts_to_iso, ts_to_iso};
mod workouts;
pub use workouts::sync_workouts;

async fn log_sync<T, C, F, Fut>(
    pool: &SqlitePool,
    data_type: &str,
    count_of: C,
    f: F,
) -> Result<T, AppError>
where
    C: Fn(&T) -> i64,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    crate::sources::sync::logged_step(pool, SourceId::WITHINGS, data_type, count_of, f).await
}

pub async fn run_sync_all(
    p: &SqlitePool,
    client: &client::WithingsClient,
    on_progress: &(dyn Fn(u32, &'static str) + Sync),
) -> SyncResult {
    let mut result = SyncResult {
        body_measures_count: 0,
        blood_pressure_count: 0,
        sleep_count: 0,
        activity_count: 0,
        vitals_count: 0,
        workouts_count: 0,
        heart_count: 0,
        devices_count: 0,
        errors: Vec::new(),
    };

    on_progress(1, "Mesures");

    match log_sync(
        p,
        "measures",
        |(b, bp, v)| (b + bp + v) as i64,
        || sync_body_measures(client, p),
    )
    .await
    {
        Ok((body_count, bp_count, vitals_count)) => {
            result.body_measures_count = body_count;
            result.blood_pressure_count = bp_count;
            result.vitals_count = vitals_count;
        }
        Err(e) => result.errors.push(format!("Mesures : {}", e)),
    }

    on_progress(2, "Activité");

    match log_sync(p, "activity", |n| *n as i64, || sync_activity(client, p)).await {
        Ok(count) => result.activity_count = count,
        Err(e) => result.errors.push(format!("Activité : {}", e)),
    }

    on_progress(3, "Sommeil");

    match log_sync(p, "sleep", |n| *n as i64, || sync_sleep(client, p)).await {
        Ok(count) => result.sleep_count = count,
        Err(e) => result.errors.push(format!("Sommeil : {}", e)),
    }

    on_progress(4, "Entraînements");

    match log_sync(p, "workouts", |n| *n as i64, || sync_workouts(client, p)).await {
        Ok(count) => result.workouts_count = count,
        Err(e) => result.errors.push(format!("Entraînements : {}", e)),
    }

    on_progress(5, "Cœur");

    match log_sync(p, "heart", |n| *n as i64, || sync_heart(client, p)).await {
        Ok(count) => result.heart_count = count,
        Err(e) => result.errors.push(format!("Cœur : {}", e)),
    }

    on_progress(6, "Appareils");

    match log_sync(p, "devices", |n| *n as i64, || sync_devices(client, p)).await {
        Ok(count) => result.devices_count = count,
        Err(e) => result.errors.push(format!("Appareils : {}", e)),
    }

    result
}

#[cfg(test)]
mod redaction_tests {
    use super::log_sync;
    use crate::error::AppError;

    #[tokio::test]
    async fn a_failed_sync_is_reported_and_logged_without_secrets() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("base en memoire");
        crate::db::bootstrap_schema(&pool).await.expect("schema");

        let res: Result<(), AppError> = log_sync(
            &pool,
            "sleep",
            |_: &()| 0,
            || async {
                Err(AppError::Internal(
                    "Token JSON illisible, body: refresh_token=RT-FUITE-5".into(),
                ))
            },
        )
        .await;

        let returned = res.unwrap_err();
        assert_eq!(returned.kind(), "Internal");
        assert!(!returned.to_string().contains("FUITE"), "{returned}");
        let logged: String =
            sqlx::query_scalar("SELECT error FROM sync_log ORDER BY rowid DESC LIMIT 1")
                .fetch_one(&pool)
                .await
                .expect("ligne de sync_log");
        assert!(!logged.contains("FUITE"), "{logged}");
    }
}
