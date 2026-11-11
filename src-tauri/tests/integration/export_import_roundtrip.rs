use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

async fn pool() -> SqlitePool {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();
    synk_lib::db::bootstrap_schema(&p).await.unwrap();
    p
}

fn write_temp_json(content: &str) -> tempfile::NamedTempFile {
    use std::io::Write;
    let mut file = tempfile::Builder::new()
        .suffix(".json")
        .tempfile()
        .expect("creation du fichier temporaire .json");
    file.write_all(content.as_bytes())
        .expect("ecriture du JSON temporaire");
    file.flush().expect("flush du JSON temporaire");
    file
}

#[tokio::test]
async fn import_restores_manual_measurements_and_goals() {
    let p = pool().await;
    let json = r#"{
      "measurement_types": [{"id": 500, "slug": "tour_cou", "label": "Tour de cou", "unit": "cm", "category": "circumference", "sort_order": 10, "is_builtin": 0}],
      "manual_measurements": [{"id": 900, "measurement_type_id": 500, "measured_at": "2026-06-01T08:00:00Z", "value": 38.0, "notes": null}],
      "goals": [{"metric_key": "weight", "target_value": 68.0, "direction": "decrease"}],
      "user_profile": [{"id": 1, "age": 40, "sex": "male", "height_cm": 178.0}]
    }"#;
    let dir = std::env::temp_dir();
    let path = dir.join("synk_test_roundtrip.json");
    std::fs::write(&path, json).unwrap();

    let report = synk_lib::commands::export::import_json_inner(&p, path.to_str().unwrap())
        .await
        .unwrap();
    assert!(report.inserted() >= 3);

    let mt: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM measurement_types_legacy WHERE slug='tour_cou'")
            .fetch_one(&p)
            .await
            .unwrap();
    assert_eq!(mt, 1);
    let mm: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM manual_measurements_legacy m JOIN measurement_types_legacy t ON t.id = m.measurement_type_id WHERE t.slug='tour_cou'",
    )
    .fetch_one(&p)
    .await
    .unwrap();
    assert_eq!(mm, 1);
    let g: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM goals WHERE metric_key='weight'")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(g, 1);
    let age: Option<i64> = sqlx::query_scalar("SELECT age FROM user_profile WHERE id=1")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(age, Some(40));

    std::fs::remove_file(&path).ok();
}

/// T-1 : round-trip body_measures — insertion via legacy, export JSON, re-import dans DB vierge.
/// Garantit que B-2 (defaut "manual" au lieu de "import") reste en non-regression :
/// une ligne sans champ "source" doit etre inseree (inserted == 1), pas silencieusement
/// ignoree a cause du CHECK source IN ('withings','manual').
#[tokio::test]
async fn roundtrip_body_measures_without_source_field() {
    let p = pool().await;

    // Insertion directe dans la table legacy (chemin reel d'ecriture)
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES ('2026-06-01T08:00:00Z', 'manual', 72.5)"
    )
    .execute(&p)
    .await
    .unwrap();

    // JSON sans champ "source" : simule un backup externe ou export partiel
    let json = r#"{
        "body_measures": [
            {"measured_at": "2026-06-02T08:00:00Z", "weight": 73.0}
        ]
    }"#;
    let file = write_temp_json(json);

    let report = synk_lib::commands::export::import_json_inner(&p, file.path().to_str().unwrap())
        .await
        .expect("import doit reussir : 'manual' est la valeur par defaut correcte");

    assert_eq!(
        report.inserted(),
        1,
        "la ligne sans source doit etre inseree (defaut 'manual')"
    );
    assert_eq!(report.ignored(), 0);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM body_measures_legacy")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(count, 2, "les 2 lignes doivent etre presentes");
}

/// T-1 : round-trip blood_pressure — meme logique que body_measures.
#[tokio::test]
async fn roundtrip_blood_pressure_without_source_field() {
    let p = pool().await;

    let json = r#"{
        "blood_pressure": [
            {"measured_at": "2026-06-01T09:00:00Z", "systolic": 120, "diastolic": 78}
        ]
    }"#;
    let file = write_temp_json(json);

    let report = synk_lib::commands::export::import_json_inner(&p, file.path().to_str().unwrap())
        .await
        .expect("import blood_pressure sans source doit reussir");

    assert_eq!(report.inserted(), 1, "inseree avec defaut 'manual'");
}

/// T-1 : round-trip sleep_records — UNIQUE(date, source).
/// Sans source, le defaut "manual" doit permettre l'insertion.
#[tokio::test]
async fn roundtrip_sleep_records_without_source_field() {
    let p = pool().await;

    let json = r#"{
        "sleep_records": [
            {"date": "2026-06-01", "duration_total": 28800, "sleep_score": 82}
        ]
    }"#;
    let file = write_temp_json(json);

    let report = synk_lib::commands::export::import_json_inner(&p, file.path().to_str().unwrap())
        .await
        .expect("import sleep_records sans source doit reussir");

    assert_eq!(report.inserted(), 1, "inseree avec defaut 'manual'");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sleep_records WHERE source='manual'")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

/// T-1 : round-trip activity — UNIQUE(date, source).
/// Sans source, le defaut "manual" doit permettre l'insertion.
#[tokio::test]
async fn roundtrip_activity_without_source_field() {
    let p = pool().await;

    let json = r#"{
        "activity": [
            {"date": "2026-06-01", "steps": 9500, "distance": 7000.0, "calories_active": 320.0}
        ]
    }"#;
    let file = write_temp_json(json);

    let report = synk_lib::commands::export::import_json_inner(&p, file.path().to_str().unwrap())
        .await
        .expect("import activity sans source doit reussir");

    assert_eq!(report.inserted(), 1, "inseree avec defaut 'manual'");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activity WHERE source='manual'")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

/// T-1 : round-trip workouts — B-2 residu.
/// La table workouts a CHECK (source IN ('withings', 'manual')).
/// Avant le fix, json_str_or(o, "source", "import") violait ce CHECK ->
/// INSERT OR IGNORE rejetait silencieusement, inserted == 0.
/// Apres le fix (defaut "manual"), la ligne est inseree.
#[tokio::test]
async fn roundtrip_workouts_without_source_field() {
    let p = pool().await;

    let json = r#"{
        "workouts": [
            {"date": "2026-06-01", "category_label": "Running", "duration": 3600}
        ]
    }"#;
    let file = write_temp_json(json);

    let report = synk_lib::commands::export::import_json_inner(&p, file.path().to_str().unwrap())
        .await
        .expect("import workout sans source doit reussir");

    assert_eq!(
        report.inserted(),
        1,
        "workout sans source doit etre insere avec defaut 'manual' (B-2)"
    );
    assert_eq!(report.ignored(), 0);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workouts WHERE source='manual'")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(count, 1, "la ligne doit etre persistee en base");
}

/// T-1 : round-trip heart_records — B-2 residu.
/// La table heart_records a CHECK (source IN ('withings', 'manual')).
/// Avant le fix, json_str_or(o, "source", "import") violait ce CHECK ->
/// INSERT OR IGNORE rejetait silencieusement, inserted == 0.
/// Apres le fix (defaut "manual"), la ligne est inseree.
#[tokio::test]
async fn roundtrip_heart_records_without_source_field() {
    let p = pool().await;

    let json = r#"{
        "heart_records": [
            {"timestamp": "2026-06-01T08:00:00Z", "heart_rate": 62}
        ]
    }"#;
    let file = write_temp_json(json);

    let report = synk_lib::commands::export::import_json_inner(&p, file.path().to_str().unwrap())
        .await
        .expect("import heart_record sans source doit reussir");

    assert_eq!(
        report.inserted(),
        1,
        "heart_record sans source doit etre insere avec defaut 'manual' (B-2)"
    );
    assert_eq!(report.ignored(), 0);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM heart_records WHERE source='manual'")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(count, 1, "la ligne doit etre persistee en base");
}

#[tokio::test]
async fn the_chosen_source_priorities_survive_an_export_and_a_restore() {
    let source = pool().await;
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES ('2026-03-01T07:00:00Z', 'withings', 80.0), ('2026-03-02T07:00:00Z', 'manual', 79.8)",
    )
    .execute(&source)
    .await
    .unwrap();
    synk_lib::services::metrics::set_priority(&source, "weight", &["withings", "manual"])
        .await
        .unwrap();
    let file = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
    synk_lib::commands::export::export_json_inner(&source, file.path())
        .await
        .unwrap();

    let restored = pool().await;
    synk_lib::commands::export::import_json_inner(&restored, file.path().to_str().unwrap())
        .await
        .unwrap();

    let order: Vec<(String, i64)> = sqlx::query_as(
        "SELECT p.source_id, p.rank FROM metric_source_priority p \
         JOIN metric_definitions d ON d.id = p.definition_id WHERE d.slug = 'weight' ORDER BY p.rank",
    )
    .fetch_all(&restored)
    .await
    .unwrap();
    assert_eq!(
        order,
        vec![("withings".to_string(), 0), ("manual".to_string(), 1)]
    );
}
