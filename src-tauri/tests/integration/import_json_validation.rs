//! Tests de validation des bornes a l'import JSON (commands/export/json.rs).
//!
//! Un backup JSON edite/forge peut injecter des valeurs hors plage physiologique
//! (weight: 99999, systolic: 9999, etc.). `import_json_inner` doit appliquer les
//! MEMES bornes que les commandes `add_manual_*` et propager une Err (rollback
//! atomique) au lieu d'inserer la valeur brute.

use sqlx::SqlitePool;
use synk_lib::commands::export::import_json_inner;
use synk_lib::db::bootstrap_schema;

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
async fn rejects_out_of_range_body_weight_and_rolls_back() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let json = r#"{
        "body_measures": [
            {"measured_at": "2026-05-01T08:00:00Z", "source": "manual", "weight": 99999.0}
        ]
    }"#;
    let file = write_temp_json(json);

    let result = import_json_inner(&pool, file.path().to_str().unwrap()).await;
    assert!(
        result.is_err(),
        "weight=99999 hors borne [0,500] doit echouer"
    );

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM body_measures_legacy")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 0, "rollback : aucune ligne ne doit etre persistee");
}

#[tokio::test]
async fn rejects_out_of_range_systolic() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let json = r#"{
        "blood_pressure": [
            {"measured_at": "2026-05-01T08:00:00Z", "source": "manual", "systolic": 9999, "diastolic": 80}
        ]
    }"#;
    let file = write_temp_json(json);

    let result = import_json_inner(&pool, file.path().to_str().unwrap()).await;
    assert!(
        result.is_err(),
        "systolic=9999 hors borne [0,300] doit echouer"
    );
}

#[tokio::test]
async fn rejects_out_of_range_steps() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let json = r#"{
        "activity": [
            {"date": "2026-05-01", "source": "manual", "steps": 999999999}
        ]
    }"#;
    let file = write_temp_json(json);

    let result = import_json_inner(&pool, file.path().to_str().unwrap()).await;
    assert!(
        result.is_err(),
        "steps=999999999 hors borne [0,200000] doit echouer"
    );
}

#[tokio::test]
async fn rejects_out_of_range_age() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let json = r#"{
        "user_profile": [
            {"id": 1, "age": 9999, "sex": "male", "height_cm": 178.0}
        ]
    }"#;
    let file = write_temp_json(json);

    let result = import_json_inner(&pool, file.path().to_str().unwrap()).await;
    assert!(result.is_err(), "age=9999 hors borne [0,150] doit echouer");
}

#[tokio::test]
async fn accepts_valid_values_across_tables() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let json = r#"{
        "body_measures": [
            {"measured_at": "2026-05-01T08:00:00Z", "source": "manual", "weight": 72.0, "bmi": 23.0}
        ],
        "blood_pressure": [
            {"measured_at": "2026-05-01T08:00:00Z", "source": "manual", "systolic": 120, "diastolic": 80, "heart_rate": 60}
        ],
        "sleep_records": [
            {"date": "2026-05-01", "source": "manual", "duration_total": 28800, "sleep_score": 85}
        ],
        "activity": [
            {"date": "2026-05-01", "source": "manual", "steps": 8000, "distance": 6000.0, "calories_active": 300.0, "calories_total": 2000.0}
        ],
        "user_profile": [
            {"id": 1, "age": 40, "sex": "male", "height_cm": 178.0}
        ]
    }"#;
    let file = write_temp_json(json);

    let report = import_json_inner(&pool, file.path().to_str().unwrap())
        .await
        .expect("des valeurs valides doivent passer la validation");
    assert_eq!(report.inserted(), 4);
    let profile = report
        .tables
        .iter()
        .find(|t| t.key == "user_profile")
        .expect("bilan du profil");
    assert_eq!(profile.updated, 1, "le profil est complete, pas insere");
}
