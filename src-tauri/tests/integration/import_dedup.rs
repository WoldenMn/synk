use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use synk_lib::commands::export::import_json_inner;
use synk_lib::db::bootstrap_schema;

// `max_connections(1)` par coherence avec `tests/export_import_roundtrip.rs`.
async fn setup_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();
    bootstrap_schema(&pool).await.unwrap();
    pool
}

// Un export reel contient l'identifiant Withings de chaque ligne : c'est lui
// qui porte la contrainte UNIQUE des quatre tables concernees.
fn backup_json() -> String {
    serde_json::json!({
        "body_measures": [{
            "measured_at": "2026-01-15T08:00:00Z", "source": "withings",
            "withings_grpid": "grp-1", "weight": 82.0
        }],
        "blood_pressure": [{
            "measured_at": "2026-01-15T08:05:00Z", "source": "withings",
            "withings_grpid": "grp-2", "systolic": 120, "diastolic": 80
        }],
        "vitals": [{
            "measured_at": "2026-01-15T08:10:00Z", "source": "withings",
            "withings_grpid": "grp-3", "vital_type": "spo2", "value": 97.0
        }],
        "workouts": [{
            "date": "2026-01-15", "source": "withings",
            "withings_id": "w-1", "category_label": "Marche", "duration": 3600
        }]
    })
    .to_string()
}

async fn count(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM {table}")))
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn importing_the_same_backup_twice_does_not_duplicate() {
    let pool = setup_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup.json");
    std::fs::write(&path, backup_json()).unwrap();
    let path_str = path.to_string_lossy().into_owned();

    let first = import_json_inner(&pool, &path_str).await.unwrap();
    assert_eq!(first.inserted(), 4, "premier import : 4 lignes inserees");
    assert_eq!(first.ignored(), 0);

    let second = import_json_inner(&pool, &path_str).await.unwrap();
    assert_eq!(second.inserted(), 0, "second import : rien de nouveau");
    assert_eq!(
        second.ignored(),
        4,
        "les 4 lignes doivent etre reconnues comme deja presentes"
    );

    for table in ["body_measures", "blood_pressure", "vitals", "workouts"] {
        assert_eq!(
            count(&pool, table).await,
            1,
            "{table} ne doit contenir qu'une ligne"
        );
    }
}

#[tokio::test]
async fn a_restored_row_is_updated_by_a_later_withings_sync() {
    let pool = setup_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup.json");
    std::fs::write(&path, backup_json()).unwrap();
    import_json_inner(&pool, &path.to_string_lossy())
        .await
        .unwrap();

    // Requete de sync reelle : elle met a jour la ligne existante grace au grpid.
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, withings_grpid, weight) \
         VALUES (?1, 'withings', 'grp-1', ?2) \
         ON CONFLICT(withings_grpid) DO UPDATE SET weight = excluded.weight",
    )
    .bind("2026-01-15T08:00:00Z")
    .bind(81.5)
    .execute(&pool)
    .await
    .unwrap();

    assert_eq!(
        count(&pool, "body_measures").await,
        1,
        "le resync du meme grpid doit mettre a jour, pas creer une seconde ligne"
    );
    let weight: f64 = sqlx::query_scalar("SELECT weight FROM body_measures")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(weight, 81.5);
}

#[tokio::test]
async fn manual_rows_without_a_withings_id_are_still_imported() {
    let pool = setup_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manual.json");
    std::fs::write(
        &path,
        serde_json::json!({
            "body_measures": [{
                "measured_at": "2026-02-01T09:00:00Z", "source": "manual", "weight": 80.0
            }]
        })
        .to_string(),
    )
    .unwrap();

    let report = import_json_inner(&pool, &path.to_string_lossy())
        .await
        .unwrap();
    assert_eq!(
        report.inserted(),
        1,
        "une ligne manuelle sans identifiant s'importe normalement"
    );
}
