use chrono::{TimeZone, Utc};
use synk_lib::services::metrics::{write_observations, ObservationWriter};
use synk_lib::sources::{Observation, SourceId};

use crate::common::memory_pool;

fn observation(metric: &str, value: f64, device: Option<&str>, id: &str) -> Observation {
    Observation {
        metric: metric.to_string(),
        measured_at: Utc.with_ymd_and_hms(2026, 10, 25, 7, 30, 0).unwrap(),
        value,
        unit: String::new(),
        source: SourceId::new("apple_health"),
        external_id: Some(id.to_string()),
        device: device.map(str::to_string),
    }
}

#[tokio::test]
async fn the_origin_device_of_an_observation_is_kept_in_its_table() {
    let pool = memory_pool().await;
    write_observations(
        &pool,
        &[
            observation("weight", 72.4, Some("Withings"), "w1"),
            observation("spo2", 97.0, Some("Oura"), "s1"),
            observation("weight", 72.0, None, "w2"),
        ],
    )
    .await
    .unwrap();

    let weights: Vec<(Option<String>,)> =
        sqlx::query_as("SELECT device FROM body_measures_legacy ORDER BY external_id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(weights, vec![(Some("Withings".to_string()),), (None,)]);
    let spo2: Option<String> =
        sqlx::query_scalar("SELECT device FROM vitals_legacy WHERE vital_type = 'spo2'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(spo2.as_deref(), Some("Oura"));
}

#[tokio::test]
async fn a_writer_inside_a_transaction_commits_nothing_when_it_is_dropped() {
    let pool = memory_pool().await;
    let mut tx = pool.begin().await.unwrap();
    let mut writer = ObservationWriter::default();
    let first = writer
        .write(&mut tx, &observation("weight", 72.4, None, "w1"))
        .await
        .unwrap();
    let again = writer
        .write(&mut tx, &observation("weight", 72.4, None, "w1"))
        .await
        .unwrap();
    assert!(first);
    assert!(!again);
    drop(tx);
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM body_measures_legacy")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0);
}

#[tokio::test]
async fn an_out_of_range_value_is_a_validation_error_not_a_write() {
    let pool = memory_pool().await;
    let mut conn = pool.acquire().await.unwrap();
    let mut writer = ObservationWriter::default();
    let err = writer
        .write(&mut conn, &observation("weight", 900.0, None, "w1"))
        .await
        .unwrap_err();
    assert!(
        matches!(err, synk_lib::error::AppError::Validation(_)),
        "{err}"
    );
}

#[tokio::test]
async fn an_api_revision_is_applied_but_a_file_writer_keeps_the_first_value() {
    let pool = memory_pool().await;
    write_observations(&pool, &[observation("weight", 72.4, None, "w1")])
        .await
        .unwrap();
    let revised = write_observations(&pool, &[observation("weight", 71.9, None, "w1")])
        .await
        .unwrap();
    assert_eq!(revised.inserted, 1);
    let stored: f64 = sqlx::query_scalar("SELECT weight FROM body_measures_legacy")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, 71.9);

    let mut conn = pool.acquire().await.unwrap();
    let mut file_writer = ObservationWriter::default();
    let kept = file_writer
        .write(&mut conn, &observation("weight", 60.0, None, "w1"))
        .await
        .unwrap();
    assert!(!kept);
    let stored: f64 = sqlx::query_scalar("SELECT weight FROM body_measures_legacy")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, 71.9);
}

#[tokio::test]
async fn a_tolerant_batch_notes_a_bad_value_and_commits_the_rest_in_one_go() {
    use synk_lib::services::metrics::write_tolerant;
    let pool = memory_pool().await;
    let batch = [
        observation("weight", 72.4, None, "w1"),
        observation("weight", 900.0, None, "w2"),
        observation("weight", 71.0, None, "w3"),
    ];

    let report = write_tolerant(&pool, &batch).await.unwrap();

    assert_eq!(report.inserted, 2);
    assert_eq!(report.notes.len(), 1);
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM body_measures_legacy")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 2);
}

#[tokio::test]
async fn an_unexpected_database_error_rolls_the_whole_batch_back() {
    use synk_lib::services::metrics::write_tolerant;
    let pool = memory_pool().await;
    sqlx::query(
        "CREATE TRIGGER boom BEFORE INSERT ON body_measures_legacy WHEN NEW.external_id = 'w3' \
         BEGIN SELECT RAISE(ABORT, 'panne simulee'); END",
    )
    .execute(&pool)
    .await
    .unwrap();
    let batch = [
        observation("weight", 72.4, None, "w1"),
        observation("weight", 72.0, None, "w2"),
        observation("weight", 71.0, None, "w3"),
    ];

    let err = write_tolerant(&pool, &batch).await.unwrap_err();

    assert!(err.to_string().contains("panne simulee"), "{err}");
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM body_measures_legacy")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0, "tout ou rien");
}
