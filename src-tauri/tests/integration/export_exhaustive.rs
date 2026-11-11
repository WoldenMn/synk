use serde_json::Value;
use std::collections::HashSet;
use synk_lib::commands::export::{
    export_json_inner, export_value, import_json_inner, EXCLUDED, EXPORTED, MAX_IMPORT_BYTES,
};

#[tokio::test]
async fn every_table_is_exported_or_excluded_with_a_reason() {
    let dir = tempfile::tempdir().unwrap();
    let pool = synk_lib::db::init_pool(dir.path().join("synk.db"))
        .await
        .expect("base temporaire initialisee");
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let covered: HashSet<&str> = EXPORTED
        .iter()
        .map(|s| s.table)
        .chain(EXCLUDED.iter().map(|(t, _)| *t))
        .collect();
    let missing: Vec<&String> = tables
        .iter()
        .filter(|t| !covered.contains(t.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "tables ni exportees ni exclues : {missing:?}"
    );
    let unknown: Vec<&&str> = covered
        .iter()
        .filter(|t| !tables.iter().any(|x| x == **t))
        .collect();
    assert!(
        unknown.is_empty(),
        "tables citees mais absentes du schema : {unknown:?}"
    );
    assert!(EXCLUDED.iter().all(|(_, why)| !why.is_empty()));
}

async fn fresh_pool(dir: &tempfile::TempDir, name: &str) -> sqlx::SqlitePool {
    synk_lib::db::init_pool(dir.path().join(name))
        .await
        .expect("base temporaire initialisee")
}

async fn populate(pool: &sqlx::SqlitePool) {
    for sql in [
        "INSERT INTO body_measures_legacy (measured_at, source, weight, fat_ratio) VALUES ('2026-06-01T07:30:00Z', 'manual', 72.4, 18.2)",
        "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic, heart_rate) VALUES ('2026-06-01T07:35:00Z', 'manual', 121, 79, 62)",
        "INSERT INTO sleep_records (date, source, duration_total, duration_deep, duration_rem, hr_average, apnea_hypopnea_index) VALUES ('2026-06-01', 'withings', 26000, 5400, 6000, 55, 3)",
        "INSERT INTO activity (date, source, steps, distance, hr_zone_0) VALUES ('2026-06-01', 'withings', 9500, 7100.0, 1200)",
        "INSERT INTO workouts (date, start_at, source, category_label, duration, hr_average, hr_zone_0) VALUES ('2026-06-01', '2026-06-01T18:00:00Z', 'manual', 'Course', 1800, 151, 300)",
        "INSERT INTO heart_records (timestamp, source, ecg_status, heart_rate, qrs_duration) VALUES ('2026-06-01T09:00:00Z', 'withings', 1, 64, 90)",
        "INSERT INTO vitals_legacy (measured_at, source, vital_type, value) VALUES ('2026-06-01T09:05:00Z', 'manual', 'spo2', 97.0)",
        "INSERT INTO measurement_types_legacy (slug, label, unit, category, sort_order, is_builtin) VALUES ('tour-de-cou-test', 'Tour de cou (test)', 'cm', 'circumference', 999, 0)",
        "INSERT INTO manual_measurements_legacy (measurement_type_id, measured_at, value) SELECT id, '2026-06-02T08:00:00Z', 38.5 FROM measurement_types_legacy WHERE slug = 'tour-de-cou-test'",
        "INSERT INTO goals (metric_key, target_value, direction) VALUES ('weight', 70.0, 'decrease')",
        "INSERT OR IGNORE INTO user_profile (id) VALUES (1)",
        "UPDATE user_profile SET age = 41, sex = 'M', height_cm = 178.0 WHERE id = 1",
        "INSERT INTO medications (name, dosage, start_date, created_at) VALUES ('Medicament test', '5 mg', '2026-05-01', '2026-05-01 10:00:00')",
        "INSERT INTO medication_logs (medication_id, taken_at) SELECT id, '2026-06-01T08:00:00Z' FROM medications WHERE name = 'Medicament test'",
        "INSERT INTO lab_results (date, test_name, value, unit) VALUES ('2026-05-20', 'Ferritine', 85.0, 'ng/mL')",
        "INSERT INTO journal_entries (date, content) VALUES ('2026-06-01', 'Bonne journee')",
        "INSERT INTO health_alerts (metric_key, condition, threshold, label, created_at) VALUES ('weight', 'above', 80.0, 'Poids haut', '2026-05-01 10:00:00')",
        "INSERT INTO alert_events (alert_id, value, triggered_at) SELECT id, 81.0, '2026-06-03T07:00:00Z' FROM health_alerts WHERE label = 'Poids haut'",
        "INSERT INTO clothing_sizes (category, value, source) VALUES ('chemise', 'M', 'manual')",
        "INSERT INTO metric_definitions (slug, label, unit, category, is_builtin) VALUES ('tour-de-taille-test', 'Tour de taille (test)', 'cm', 'custom', 0)",
        "INSERT INTO metric_source_priority (definition_id, source_id, rank) SELECT id, 'manual', 0 FROM metric_definitions WHERE slug = 'weight'",
        "INSERT INTO metrics (definition_id, measured_at, value, source) SELECT id, '2026-06-04T08:00:00Z', 80.5, 'manual' FROM metric_definitions WHERE slug = 'tour-de-taille-test'",
    ] {
        sqlx::query(sql)
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("{sql} : {e}"));
    }
}

fn user_rows(mut v: Value) -> Value {
    for table in ["measurement_types", "metric_definitions"] {
        if let Some(rows) = v.get_mut(table).and_then(Value::as_array_mut) {
            rows.retain(|r| r.get("is_builtin").and_then(Value::as_i64) == Some(0));
        }
    }
    v
}

#[tokio::test]
async fn export_then_import_into_an_empty_base_gives_the_same_data() {
    let dir = tempfile::tempdir().unwrap();
    let source = fresh_pool(&dir, "source.db").await;
    populate(&source).await;
    let file = dir.path().join("export.json");
    export_json_inner(&source, &file).await.unwrap();

    let target = fresh_pool(&dir, "cible.db").await;
    let report = import_json_inner(&target, file.to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(
        report.tables.iter().map(|t| t.orphans).sum::<u64>(),
        0,
        "{report:?}"
    );
    assert_eq!(
        user_rows(export_value(&source).await.unwrap()),
        user_rows(export_value(&target).await.unwrap())
    );
}

#[tokio::test]
async fn every_exported_table_holds_a_row_in_the_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let source = fresh_pool(&dir, "source.db").await;
    populate(&source).await;
    let json = export_value(&source).await.unwrap();
    for spec in EXPORTED {
        let rows = json[spec.key]
            .as_array()
            .unwrap_or_else(|| panic!("{} absente", spec.key));
        assert!(!rows.is_empty(), "{} vide dans l'export de test", spec.key);
    }
}

#[tokio::test]
async fn importing_the_same_file_twice_adds_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let source = fresh_pool(&dir, "source.db").await;
    populate(&source).await;
    let file = dir.path().join("export.json");
    export_json_inner(&source, &file).await.unwrap();
    let target = fresh_pool(&dir, "cible.db").await;
    import_json_inner(&target, file.to_str().unwrap())
        .await
        .unwrap();

    let second = import_json_inner(&target, file.to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(second.inserted(), 0, "{second:?}");
}

#[tokio::test]
async fn a_2_15_export_imports_without_loss_or_duplicate() {
    let dir = tempfile::tempdir().unwrap();
    let target = fresh_pool(&dir, "cible.db").await;
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/export_2_15.json"
    );

    import_json_inner(&target, fixture).await.unwrap();
    let again = import_json_inner(&target, fixture).await.unwrap();

    let (slug, value): (String, f64) = sqlx::query_as(
        "SELECT t.slug, m.value FROM manual_measurements_legacy m JOIN measurement_types_legacy t ON t.id = m.measurement_type_id",
    )
    .fetch_one(&target)
    .await
    .unwrap();
    assert_eq!((slug.as_str(), value), ("tour-de-cou-test", 38.5));
    assert_eq!(again.inserted(), 0, "{again:?}");
    let (age,): (Option<i64>,) = sqlx::query_as("SELECT age FROM user_profile WHERE id = 1")
        .fetch_one(&target)
        .await
        .unwrap();
    assert_eq!(age, Some(41));
}

#[tokio::test]
async fn a_2_15_export_with_zeroed_external_ids_keeps_every_row() {
    let dir = tempfile::tempdir().unwrap();
    let target = fresh_pool(&dir, "cible.db").await;
    let file = dir.path().join("deux_pesees.json");
    std::fs::write(
        &file,
        r#"{"body_measures": [
            {"id": 1, "measured_at": "2026-06-01T07:30:00Z", "source": "withings", "withings_grpid": 0, "weight": 72.4},
            {"id": 2, "measured_at": "2026-06-02T07:30:00Z", "source": "withings", "withings_grpid": 0, "weight": 72.1}
        ]}"#,
    )
    .unwrap();

    let report = import_json_inner(&target, file.to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(report.inserted(), 2, "{report:?}");
}

#[tokio::test]
async fn the_profile_is_completed_never_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let target = fresh_pool(&dir, "cible.db").await;
    sqlx::query("INSERT OR IGNORE INTO user_profile (id) VALUES (1)")
        .execute(&target)
        .await
        .unwrap();
    sqlx::query("UPDATE user_profile SET age = 35 WHERE id = 1")
        .execute(&target)
        .await
        .unwrap();
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/export_2_15.json"
    );

    import_json_inner(&target, fixture).await.unwrap();

    let (age, height): (Option<i64>, Option<f64>) =
        sqlx::query_as("SELECT age, height_cm FROM user_profile WHERE id = 1")
            .fetch_one(&target)
            .await
            .unwrap();
    assert_eq!((age, height), (Some(35), Some(178.0)));
}

#[tokio::test]
async fn an_oversized_file_is_refused_before_being_read() {
    let dir = tempfile::tempdir().unwrap();
    let target = fresh_pool(&dir, "cible.db").await;
    let file = dir.path().join("enorme.json");
    std::fs::File::create(&file)
        .unwrap()
        .set_len(MAX_IMPORT_BYTES + 1)
        .unwrap();

    let err = import_json_inner(&target, file.to_str().unwrap())
        .await
        .unwrap_err();

    assert!(err.detail().contains("trop volumineux"), "obtenu : {err}");
}

#[tokio::test]
async fn a_file_that_is_not_a_json_object_is_refused_on_its_first_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let target = fresh_pool(&dir, "cible.db").await;
    let file = dir.path().join("pas-un-objet.json");
    let mut body = b"MZ".to_vec();
    body.resize(1024 * 1024, b'x');
    std::fs::write(&file, body).unwrap();

    let err = import_json_inner(&target, file.to_str().unwrap())
        .await
        .unwrap_err();

    assert_eq!(err.kind(), "Validation", "obtenu : {err}");
    assert!(
        err.detail().contains("Format JSON invalide"),
        "obtenu : {err}"
    );
}

#[tokio::test]
async fn a_json_object_after_leading_blanks_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let target = fresh_pool(&dir, "cible.db").await;
    let file = dir.path().join("blancs.json");
    std::fs::write(&file, "  \n\t{}").unwrap();

    let outcome = import_json_inner(&target, file.to_str().unwrap()).await;

    assert!(
        outcome.is_ok() || outcome.unwrap_err().detail() != "Format JSON invalide",
        "un objet precede de blancs n'est pas refuse sur sa premiere lettre"
    );
}

#[tokio::test]
async fn a_null_column_is_exported_as_null_not_zero() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let pool = fresh_pool(&dir, "null.db").await;
    sqlx::query("INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES ('2026-06-01T07:30:00Z', 'manual', 72.4)")
        .execute(&pool)
        .await
        .expect("pesee");
    let json = export_value(&pool).await.expect("export");
    let row = &json["body_measures"][0];
    assert_eq!(row["weight"], 72.4);
    assert!(row["fat_mass"].is_null(), "{row}");
    assert!(row["withings_grpid"].is_null(), "{row}");
}

#[tokio::test]
async fn an_export_carries_no_row_id_so_an_import_keeps_the_rows_already_there() {
    let dir = tempfile::tempdir().unwrap();
    let source = fresh_pool(&dir, "source.db").await;
    populate(&source).await;
    let json = export_value(&source).await.unwrap();
    for spec in EXPORTED {
        for row in json[spec.key].as_array().unwrap() {
            assert!(
                row.get("id").is_none(),
                "{} exporte un id : {row}",
                spec.key
            );
        }
    }

    let target = fresh_pool(&dir, "cible.db").await;
    sqlx::query("INSERT INTO body_measures_legacy (id, measured_at, source, weight) VALUES (1, '2026-03-01T07:30:00Z', 'manual', 90.0)")
        .execute(&target)
        .await
        .unwrap();
    let file = dir.path().join("export.json");
    export_json_inner(&source, &file).await.unwrap();

    import_json_inner(&target, file.to_str().unwrap())
        .await
        .unwrap();

    let (rows,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM body_measures_legacy")
        .fetch_one(&target)
        .await
        .unwrap();
    assert_eq!(rows, 2);
}

#[tokio::test]
async fn a_profile_with_nothing_to_complete_is_counted_as_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let target = fresh_pool(&dir, "cible.db").await;
    sqlx::query("INSERT OR IGNORE INTO user_profile (id) VALUES (1)")
        .execute(&target)
        .await
        .unwrap();
    sqlx::query("UPDATE user_profile SET age = 35 WHERE id = 1")
        .execute(&target)
        .await
        .unwrap();
    let file = dir.path().join("profil.json");
    std::fs::write(&file, r#"{"user_profile": [{"id": 1, "age": 40}]}"#).unwrap();

    let report = import_json_inner(&target, file.to_str().unwrap())
        .await
        .unwrap();

    let profile = report
        .tables
        .iter()
        .find(|t| t.key == "user_profile")
        .unwrap();
    assert_eq!((profile.ignored, profile.updated), (1, 0), "{report:?}");
}

#[tokio::test]
async fn a_row_whose_parent_is_unknown_is_counted_as_an_orphan() {
    let dir = tempfile::tempdir().unwrap();
    let target = fresh_pool(&dir, "cible.db").await;
    let file = dir.path().join("orphelines.json");
    std::fs::write(
        &file,
        r#"{"manual_measurements": [
            {"measurement_type": {"slug": "inconnu"}, "measured_at": "2026-06-02T08:00:00Z", "value": 38.5}
        ]}"#,
    )
    .unwrap();

    let report = import_json_inner(&target, file.to_str().unwrap())
        .await
        .unwrap();

    let table = report
        .tables
        .iter()
        .find(|t| t.key == "manual_measurements")
        .unwrap();
    assert_eq!((table.orphans, table.inserted), (1, 0), "{report:?}");
}
