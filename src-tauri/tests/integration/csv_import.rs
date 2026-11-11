use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use sqlx::SqlitePool;
use synk_lib::sources::import::pipeline::ImportReport;
use synk_lib::sources::import::table::store;
use synk_lib::sources::import::table::{import_csv, inspect_file, CsvConfig};

use crate::common::memory_pool;
use crate::support::activity_files::write;
use crate::support::import_gate;

const SCALE: &str = "Date;Heure;Poids (kg);Masse grasse (%);Tension sys;Tension dia\n25/10/2026;07:30;72,4;18,5;121;79\n26/10/2026;07:41;72,1;18,4;118;76\n27/10/2026;07:35;;;;\n";

const LIBRE: &str = "Patient report,Imprime le 2026-10-25,,,,,\nDevice,Serial Number,Device Timestamp,Record Type,Historic Glucose mg/dL,Scan Glucose mg/dL,Strip Glucose mg/dL\nFreeStyle Libre,SYN001,25-10-2026 07:30,0,99,,\nFreeStyle Libre,SYN001,25-10-2026 07:45,0,108,,\nFreeStyle Libre,SYN001,25-10-2026 08:02,1,,126,\n";

async fn run(
    pool: &SqlitePool,
    path: &Path,
    config: CsvConfig,
) -> Result<ImportReport, synk_lib::error::AppError> {
    import_csv(
        pool,
        path.to_str().unwrap(),
        config,
        Arc::new(AtomicBool::new(false)),
        &|_| {},
    )
    .await
}

async fn count(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT COUNT(*) FROM {table} WHERE source = 'csv'"
    )))
    .fetch_one(pool)
    .await
    .unwrap()
}

fn suggested(path: &Path) -> CsvConfig {
    inspect_file(path.to_str().unwrap())
        .expect("inspection")
        .suggestion
}

#[tokio::test]
async fn the_assistant_suggestion_imports_a_scale_and_pressure_file() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "balance.csv", SCALE.as_bytes());

    let report = run(&pool, &path, suggested(&path)).await.expect("import");

    assert_eq!(report.tally.observations, 4);
    assert_eq!(report.tally.blood_pressure, 2);
    assert_eq!(report.skipped.len(), 0);
    let weights: Vec<f64> = sqlx::query_scalar(
        "SELECT weight FROM body_measures_legacy WHERE source = 'csv' AND weight IS NOT NULL ORDER BY measured_at",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(weights, vec![72.4, 72.1]);
    let bp: (i64, i64) = sqlx::query_as(
        "SELECT systolic, diastolic FROM blood_pressure WHERE source = 'csv' ORDER BY measured_at LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(bp, (121, 79));
}

#[tokio::test]
async fn a_libreview_export_is_recognised_and_its_values_converted() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "libreview.csv", LIBRE.as_bytes());
    let inspection = inspect_file(path.to_str().unwrap()).expect("inspection");
    assert_eq!(inspection.preset, Some("LibreView"));

    let report = run(&pool, &path, inspection.suggestion)
        .await
        .expect("import");

    assert_eq!(report.tally.observations, 3);
    let values: Vec<f64> = sqlx::query_scalar(
        "SELECT value FROM vitals_legacy WHERE source = 'csv' AND vital_type = 'blood_glucose' ORDER BY measured_at",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(values.len(), 3);
    assert!((values[0] - 5.4955).abs() < 1e-3, "{values:?}");
    assert!((values[2] - 6.994).abs() < 1e-3, "{values:?}");
}

#[tokio::test]
async fn importing_the_same_csv_twice_adds_nothing_and_an_overlapping_one_adds_only_the_new_rows() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "balance.csv", SCALE.as_bytes());
    let config = suggested(&path);
    run(&pool, &path, config.clone()).await.expect("premier");
    let before = (
        count(&pool, "body_measures_legacy").await,
        count(&pool, "blood_pressure").await,
    );

    let again = run(&pool, &path, config.clone()).await.expect("second");

    assert_eq!(
        (again.tally.observations, again.tally.blood_pressure),
        (0, 0)
    );
    assert_eq!(again.tally.observations_ignored, 4);
    assert_eq!(
        (
            count(&pool, "body_measures_legacy").await,
            count(&pool, "blood_pressure").await
        ),
        before
    );

    let longer = format!("{SCALE}28/10/2026;07:50;71,9;18,3;120;78\n");
    let bigger = write(dir.path(), "balance-plus.csv", longer.as_bytes());
    let grown = run(&pool, &bigger, config).await.expect("recouvrement");
    assert_eq!(grown.tally.observations, 2);
    assert_eq!(grown.tally.blood_pressure, 1);
}

#[tokio::test]
async fn a_binary_file_or_a_wrong_mapping_is_a_named_error_and_writes_nothing() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "balance.csv", SCALE.as_bytes());
    let mut config = suggested(&path);

    let binary = write(dir.path(), "faux.csv", b"PK\x03\x04\x00\x00\x00binaire");
    let err = run(&pool, &binary, config.clone())
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("binaires"), "{err}");

    config.columns[0].column = "Colonne absente".into();
    let err = run(&pool, &path, config).await.unwrap_err().to_string();
    assert!(err.contains("introuvable"), "{err}");

    let cut = write(
        dir.path(),
        "coupe.csv",
        &SCALE.as_bytes()[..SCALE.len() - 150],
    );
    let err = inspect_file(cut.to_str().unwrap()).unwrap_err().to_string();
    assert!(err.contains("vide"), "{err}");

    assert_eq!(count(&pool, "body_measures_legacy").await, 0);
    assert_eq!(count(&pool, "blood_pressure").await, 0);
}

#[tokio::test]
async fn a_row_cut_in_the_middle_is_counted_not_guessed() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let kept: Vec<&str> = SCALE.lines().take(3).collect();
    let cut_text = format!("{}\n28/10/2026 07:", kept.join("\n"));
    let path = write(dir.path(), "coupe.csv", cut_text.as_bytes());
    let whole = write(dir.path(), "entier.csv", SCALE.as_bytes());

    let report = run(&pool, &path, suggested(&whole)).await.expect("import");

    assert_eq!(report.tally.observations, 4);
    assert_eq!(report.skipped.len(), 1);
    assert_eq!(report.skipped[0].reason, "ligne : date illisible");
}

#[tokio::test]
async fn a_cancelled_csv_import_writes_nothing() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "balance.csv", SCALE.as_bytes());

    let report = import_csv(
        &pool,
        path.to_str().unwrap(),
        suggested(&path),
        Arc::new(AtomicBool::new(true)),
        &|_| {},
    )
    .await
    .expect("import");

    assert!(report.cancelled);
    assert_eq!(count(&pool, "body_measures_legacy").await, 0);
}

#[tokio::test]
async fn a_saved_mapping_comes_back_for_the_same_columns_and_can_be_replaced_or_removed() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "balance.csv", SCALE.as_bytes());
    let inspection = inspect_file(path.to_str().unwrap()).expect("inspection");

    assert!(store::matching(&pool, &inspection.signature)
        .await
        .unwrap()
        .is_empty());
    let id = store::save(
        &pool,
        "Ma balance",
        &inspection.signature,
        &inspection.suggestion,
    )
    .await
    .unwrap();
    let found = store::matching(&pool, &inspection.signature).await.unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].config, inspection.suggestion);

    let mut changed = inspection.suggestion.clone();
    changed.decimal = '.';
    let same = store::save(&pool, "Ma balance", &inspection.signature, &changed)
        .await
        .unwrap();
    assert_eq!(same, id);
    let found = store::matching(&pool, &inspection.signature).await.unwrap();
    assert_eq!((found.len(), found[0].config.decimal), (1, '.'));

    let other = write(dir.path(), "autre.csv", b"Jour;Valeur\n1;2\n");
    let other_inspection = inspect_file(other.to_str().unwrap()).expect("inspection");
    assert!(store::matching(&pool, &other_inspection.signature)
        .await
        .unwrap()
        .is_empty());

    assert!(store::remove(&pool, id).await.unwrap());
    assert!(!store::remove(&pool, id).await.unwrap());
    assert!(store::save(&pool, "  ", &inspection.signature, &changed)
        .await
        .is_err());
}

#[tokio::test]
async fn a_mapping_that_could_never_import_is_refused_when_saved() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "balance.csv", SCALE.as_bytes());
    let inspection = inspect_file(path.to_str().unwrap()).expect("inspection");

    for broken in [
        {
            let mut c = inspection.suggestion.clone();
            c.delimiter = 'x';
            c
        },
        {
            let mut c = inspection.suggestion.clone();
            c.date_format = "inconnu".into();
            c
        },
        {
            let mut c = inspection.suggestion.clone();
            c.columns.clear();
            c
        },
        {
            let mut c = inspection.suggestion.clone();
            c.date_column = "d".repeat(500);
            c
        },
    ] {
        let err = store::save(&pool, "Cassee", &inspection.signature, &broken)
            .await
            .unwrap_err();
        assert_eq!(err.kind(), "Validation", "{err}");
    }
    assert!(store::matching(&pool, &inspection.signature)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn an_unreadable_stored_mapping_is_skipped_but_leaves_a_warning_trace() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    sqlx::query("INSERT INTO csv_mappings (name, signature, config) VALUES ('Abimee', 'sig', 'pas du json')")
        .execute(&pool)
        .await
        .unwrap();
    let found = store::matching(&pool, "sig").await.unwrap();
    assert!(found.is_empty());
    let unreadable = store::unreadable_count(&pool, "sig").await.unwrap();
    assert_eq!(unreadable, 1);
}
