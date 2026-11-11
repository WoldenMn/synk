use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use sqlx::SqlitePool;
use synk_lib::sources::import::pipeline::ImportReport;
use synk_lib::sources::import::trace::import_trace;

use crate::common::memory_pool;
use crate::support::activity_files::{sample_gpx, sample_tcx, write};
use crate::support::import_gate;

async fn import(pool: &SqlitePool, path: &Path) -> Result<ImportReport, synk_lib::error::AppError> {
    import_trace(
        pool,
        path.to_str().unwrap(),
        Arc::new(AtomicBool::new(false)),
        &|_| {},
    )
    .await
}

async fn workouts(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM workouts WHERE source = 'trace'")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn the_trace_source_exists_after_the_migration() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let label: String = sqlx::query_scalar("SELECT label FROM sources WHERE id = 'trace'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(label, "Traces GPX et TCX");
}

#[tokio::test]
async fn a_gpx_track_becomes_a_workout_with_distance_climb_and_heart_rate() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "sortie.gpx", sample_gpx().as_bytes());

    let report = import(&pool, &path).await.expect("import");

    assert_eq!(report.tally.workouts, 1);
    let row: (String, i64, f64, f64, i64, i64, i64, String) = sqlx::query_as(
        "SELECT category_label, duration, distance, elevation, hr_average, hr_min, hr_max, device FROM workouts WHERE source = 'trace'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "Velo");
    assert_eq!(row.1, 3480);
    assert!((row.2 - 2579.7).abs() < 2.0, "{}", row.2);
    assert_eq!(row.3, 100.0);
    assert_eq!((row.4, row.5, row.6), (135, 120, 149));
    assert_eq!(row.7, "Montre synthetique");
}

#[tokio::test]
async fn a_tcx_activity_takes_its_totals_from_the_laps() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "sortie.tcx", sample_tcx().as_bytes());

    let report = import(&pool, &path).await.expect("import");

    assert_eq!(report.tally.workouts, 1);
    let row: (String, i64, f64, f64, f64, i64, String) = sqlx::query_as(
        "SELECT category_label, duration, distance, calories, elevation, hr_max, device FROM workouts WHERE source = 'trace'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "Course");
    assert_eq!(row.1, 1080);
    assert_eq!((row.2, row.3, row.4, row.5), (5400.0, 260.0, 27.0, 146));
    assert_eq!(row.6, "Compteur synthetique");
}

#[tokio::test]
async fn reimporting_a_gpx_or_a_tcx_adds_nothing() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let gpx = write(dir.path(), "a.gpx", sample_gpx().as_bytes());
    let tcx = write(dir.path(), "b.tcx", sample_tcx().as_bytes());
    import(&pool, &gpx).await.expect("gpx");
    import(&pool, &tcx).await.expect("tcx");
    assert_eq!(workouts(&pool).await, 2);

    let again_gpx = import(&pool, &gpx).await.expect("gpx");
    let again_tcx = import(&pool, &tcx).await.expect("tcx");

    assert_eq!(workouts(&pool).await, 2);
    assert_eq!(again_gpx.tally.workouts + again_tcx.tally.workouts, 0);
}

#[tokio::test]
async fn a_truncated_or_foreign_file_is_a_named_error_and_writes_nothing() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let gpx = sample_gpx();
    let cut = write(
        dir.path(),
        "coupe.gpx",
        &gpx.as_bytes()[..gpx.find("</trkseg>").unwrap()],
    );
    let err = import(&pool, &cut).await.unwrap_err().to_string();
    assert!(err.contains("GPX") && err.contains("tronqué"), "{err}");

    let tcx = sample_tcx();
    let broken = write(dir.path(), "casse.tcx", &tcx.as_bytes()[..tcx.len() / 2]);
    let err = import(&pool, &broken).await.unwrap_err().to_string();
    assert!(err.contains("TCX"), "{err}");

    let foreign = write(dir.path(), "autre.gpx", b"{\"json\": true}");
    let err = import(&pool, &foreign).await.unwrap_err().to_string();
    assert!(err.contains("pas un gpx"), "{err}");

    assert_eq!(workouts(&pool).await, 0);
}

#[tokio::test]
async fn a_cancelled_trace_import_writes_nothing() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "a.gpx", sample_gpx().as_bytes());
    let cancel = Arc::new(AtomicBool::new(true));

    let report = import_trace(&pool, path.to_str().unwrap(), cancel, &|_| {})
        .await
        .expect("import");

    assert_eq!(workouts(&pool).await, 0);
    assert!(report.cancelled);
}
