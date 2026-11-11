use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use chrono::{DateTime, Duration, Local, Utc};
use sqlx::SqlitePool;
use synk_lib::sources::import::fit::import_fit;
use synk_lib::sources::import::pipeline::ImportReport;

use crate::common::memory_pool;
use crate::support::activity_files::{sample_fit, write, NIGHT_START};
use crate::support::import_gate;

async fn import(pool: &SqlitePool, path: &Path) -> Result<ImportReport, synk_lib::error::AppError> {
    import_fit(
        pool,
        path.to_str().unwrap(),
        Arc::new(AtomicBool::new(false)),
        &|_| {},
    )
    .await
}

type BodyRow = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
);

const TABLES: [&str; 6] = [
    "body_measures_legacy",
    "vitals_legacy",
    "blood_pressure",
    "sleep_records",
    "activity",
    "workouts",
];

async fn counts(pool: &SqlitePool) -> Vec<i64> {
    let mut all = Vec::new();
    for t in TABLES {
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM {t} WHERE source = 'fit'"
        )))
        .fetch_one(pool)
        .await
        .unwrap();
        all.push(n);
    }
    all
}

fn expected_night() -> String {
    let start: DateTime<Utc> = DateTime::from_timestamp(NIGHT_START, 0).unwrap();
    (start.with_timezone(&Local).naive_local() + Duration::hours(6))
        .format("%Y-%m-%d")
        .to_string()
}

#[tokio::test]
async fn a_fit_file_lands_in_the_right_tables_with_its_device() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "sortie.fit", &sample_fit());

    let report = import(&pool, &path).await.expect("import");

    assert!(!report.cancelled);
    assert_eq!(counts(&pool).await, vec![6, 0, 1, 1, 0, 1]);
    assert_eq!(report.tally.workouts, 1);
    assert_eq!(report.tally.blood_pressure, 1);
    assert_eq!(report.tally.sleep, 1);
    assert_eq!(report.tally.observations, 6);
    assert_eq!(report.tally.devices.get("Garmin Forerunner Test"), Some(&9));

    let workout: (String, i64, f64, f64, f64, i64, i64, i64, String) = sqlx::query_as(
        "SELECT category_label, duration, distance, calories, elevation, hr_average, hr_min, hr_max, start_at FROM workouts WHERE source = 'fit'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(workout.0, "Course");
    assert_eq!(workout.1, 1800);
    assert!((workout.2 - 5123.45).abs() < 0.01, "{}", workout.2);
    assert_eq!(
        (workout.3, workout.4, workout.5, workout.6, workout.7),
        (320.0, 86.0, 148, 102, 176)
    );
    assert_eq!(workout.8, "2026-10-25T10:00:00Z");

    let weights: Vec<BodyRow> =
        sqlx::query_as(
            "SELECT weight, fat_ratio, hydration, muscle_mass, bone_mass, bmi FROM body_measures_legacy WHERE source = 'fit' ORDER BY id",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
    let flat: Vec<f64> = weights
        .iter()
        .flat_map(|r| [r.0, r.1, r.2, r.3, r.4, r.5])
        .flatten()
        .collect();
    assert_eq!(flat.len(), 6);
    let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
    assert!(flat.iter().any(|v| near(*v, 72.40)), "{flat:?}");
    assert!(flat.iter().any(|v| near(*v, 18.50)), "{flat:?}");
    assert!(flat.iter().any(|v| near(*v, 39.82)), "{flat:?}");
    assert!(flat.iter().any(|v| near(*v, 54.80)), "{flat:?}");
    assert!(flat.iter().any(|v| near(*v, 3.10)), "{flat:?}");
    assert!(flat.iter().any(|v| near(*v, 23.10)), "{flat:?}");

    let bp: (i64, i64) =
        sqlx::query_as("SELECT systolic, diastolic FROM blood_pressure WHERE source = 'fit'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(bp, (121, 79));

    let sleep: (String, i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT date, duration_total, duration_light, duration_deep, duration_rem, duration_awake FROM sleep_records WHERE source = 'fit'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        sleep,
        (expected_night(), 24_600, 15_600, 5_400, 3_600, 1_200)
    );
}

#[tokio::test]
async fn importing_the_same_fit_file_twice_adds_nothing() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "sortie.fit", &sample_fit());
    import(&pool, &path).await.expect("premier import");
    let first = counts(&pool).await;

    let again = import(&pool, &path).await.expect("second import");

    assert_eq!(counts(&pool).await, first);
    assert_eq!(again.tally.workouts, 0);
    assert_eq!(again.tally.blood_pressure, 0);
    assert_eq!(again.tally.observations, 0);
    assert_eq!(again.tally.observations_ignored, 6);
}

#[tokio::test]
async fn a_corrupted_or_truncated_fit_file_is_a_named_error_and_writes_nothing() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let good = sample_fit();

    let mut flipped = good.clone();
    let middle = flipped.len() / 2;
    flipped[middle] ^= 0xFF;
    let corrupted = write(dir.path(), "corrompu.fit", &flipped);
    let err = import(&pool, &corrupted).await.unwrap_err().to_string();
    assert!(err.contains("FIT"), "{err}");

    let cut = write(dir.path(), "tronque.fit", &good[..good.len() - 40]);
    let err = import(&pool, &cut).await.unwrap_err().to_string();
    assert!(err.contains("FIT"), "{err}");

    let junk = write(
        dir.path(),
        "autre.fit",
        b"ceci n'est pas un fichier FIT du tout",
    );
    let err = import(&pool, &junk).await.unwrap_err().to_string();
    assert!(err.contains("FIT"), "{err}");

    let empty = write(dir.path(), "vide.fit", b"");
    assert!(import(&pool, &empty).await.is_err());

    assert_eq!(counts(&pool).await, vec![0; 6]);
}

#[tokio::test]
async fn a_bad_checksum_says_so() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let mut bytes = sample_fit();
    let last = bytes.len() - 1;
    bytes[last] ^= 0x01;
    let path = write(dir.path(), "crc.fit", &bytes);

    let err = import(&pool, &path).await.unwrap_err().to_string();

    assert!(err.contains("somme de contrôle"), "{err}");
}

#[tokio::test]
async fn a_cancelled_fit_import_writes_nothing() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "sortie.fit", &sample_fit());

    let report = import_fit(
        &pool,
        path.to_str().unwrap(),
        Arc::new(AtomicBool::new(true)),
        &|_| {},
    )
    .await
    .expect("import");

    assert!(report.cancelled);
    assert_eq!(counts(&pool).await, vec![0; 6]);
}

#[tokio::test]
async fn a_file_with_another_extension_is_refused() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "sortie.txt", &sample_fit());
    assert!(import(&pool, &path).await.is_err());
}
