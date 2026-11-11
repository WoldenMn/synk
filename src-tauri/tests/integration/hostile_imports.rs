use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use sqlx::SqlitePool;
use synk_lib::sources::import::apple_health::run::import_apple_health_with;
use synk_lib::sources::import::apple_health::{import_apple_health, ImportReport};
use synk_lib::sources::import::limits::Limits;
use synk_lib::sources::import::trace::import_trace;

use crate::common::memory_pool;
use crate::support::apple_export::{record, small_export, stamp};
use crate::support::import_gate;

const TABLES: [&str; 6] = [
    "body_measures_legacy",
    "vitals_legacy",
    "blood_pressure",
    "sleep_records",
    "activity",
    "workouts",
];

async fn rows_written(pool: &SqlitePool) -> i64 {
    let mut total = 0;
    for table in TABLES {
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM {table} WHERE source IN ('apple_health', 'trace')"
        )))
        .fetch_one(pool)
        .await
        .unwrap();
        total += n;
    }
    total
}

fn valid_records(count: usize) -> String {
    (0..count)
        .map(|i| record("HeartRate", "count/min", "62", "Apple Watch", &stamp(i)))
        .collect()
}

async fn apple(pool: &SqlitePool, path: &Path) -> Result<ImportReport, synk_lib::error::AppError> {
    import_apple_health(
        pool,
        path.to_str().unwrap(),
        Arc::new(AtomicBool::new(false)),
        &|_| {},
    )
    .await
}

fn write_file(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

#[tokio::test]
async fn a_zip_bomb_is_refused_and_nothing_is_written() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bombe.zip");
    {
        let mut zip = zip::ZipWriter::new(BufWriter::new(std::fs::File::create(&path).unwrap()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("apple_health_export/export.xml", options)
            .unwrap();
        let block = vec![b' '; 1024 * 1024];
        for _ in 0..64 {
            zip.write_all(&block).unwrap();
        }
        zip.finish().unwrap();
    }
    let compressed = std::fs::metadata(&path).unwrap().len();
    assert!(compressed < 200 * 1024, "{compressed}");

    let limits = Limits {
        bytes: 16 * 1024 * 1024,
        ..Limits::APPLE_HEALTH
    };
    let err = import_apple_health_with(
        &pool,
        path.to_str().unwrap(),
        limits,
        Arc::new(AtomicBool::new(false)),
        &|_| {},
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("limite"), "{err}");
    assert_eq!(rows_written(&pool).await, 0);
}

#[tokio::test]
async fn an_apple_attribute_of_several_megabytes_is_refused_and_the_valid_records_before_it_are_rolled_back(
) {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let mut body = valid_records(500);
    body += &record(
        "HeartRate",
        "count/min",
        "62",
        &"x".repeat(6 * 1024 * 1024),
        &stamp(9999),
    );
    let path = small_export(dir.path(), &body);
    let err = apple(&pool, &path).await.unwrap_err();
    assert!(err.to_string().contains("volumineux"), "{err}");
    assert_eq!(rows_written(&pool).await, 0);
}

#[tokio::test]
async fn a_gpx_with_a_giant_text_node_is_refused_and_nothing_is_written() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let gpx = format!(
        "<?xml version=\"1.0\"?><gpx version=\"1.1\" creator=\"x\"><trk><name>{}</name><trkseg></trkseg></trk></gpx>",
        "x".repeat(6 * 1024 * 1024)
    );
    let path = write_file(dir.path(), "bombe.gpx", &gpx);
    let err = import_trace(
        &pool,
        path.to_str().unwrap(),
        Arc::new(AtomicBool::new(false)),
        &|_| {},
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("volumineux"), "{err}");
    assert_eq!(rows_written(&pool).await, 0);
}

#[tokio::test]
async fn ten_thousand_distinct_units_and_devices_stay_bounded_in_the_report_and_the_column() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let mut body = String::new();
    for i in 0..10_000 {
        body += &record("BodyMass", &format!("u{i}"), "62", "Apple Watch", &stamp(i));
    }
    for i in 0..10_000 {
        body += &record(
            "BodyMass",
            "kg",
            "70",
            &format!("{}-{i}", "appareil".repeat(100)),
            &stamp(20_000 + i),
        );
    }
    let report = apple(&pool, &small_export(dir.path(), &body))
        .await
        .unwrap();
    assert!(report.skipped.len() <= 51, "{}", report.skipped.len());
    assert_eq!(report.skipped.iter().map(|s| s.count).sum::<u64>(), 10_000);
    assert!(
        report.tally.devices.len() <= 51,
        "{}",
        report.tally.devices.len()
    );
    assert!(report.tally.devices.keys().all(|d| d.chars().count() <= 80));
    let longest: i64 = sqlx::query_scalar(
        "SELECT MAX(LENGTH(device)) FROM body_measures_legacy WHERE source = 'apple_health'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(longest <= 80, "{longest}");
}
