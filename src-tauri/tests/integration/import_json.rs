//! Tests de regression pour `import_json_inner` (commands/export/json.rs).
//!
//! Avant le fix : chaque INSERT utilisait `let _ = ...execute().await;` (erreur
//! avalee) + `imported += 1;` inconditionnel -> faux succes. Ces tests verrouillent
//! le comportement honnete : transaction atomique, erreurs remontees, compteur
//! distinguant lignes inserees vs ignorees (doublons).

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
async fn inner_counts_inserted_and_ignored_duplicates() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    // Ligne pre-existante : (date, source) est la cle UNIQUE de sleep_records.
    sqlx::query(
        "INSERT INTO sleep_records (date, source, sleep_score) VALUES ('2026-05-01', 'manual', 80)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let json = r#"{
        "sleep_records": [
            {"date": "2026-05-01", "source": "manual", "sleep_score": 80},
            {"date": "2026-05-02", "source": "manual", "sleep_score": 70}
        ]
    }"#;
    let file = write_temp_json(json);

    let report = import_json_inner(&pool, file.path().to_str().unwrap())
        .await
        .expect("import doit reussir");

    assert_eq!(report.inserted(), 1, "seule 2026-05-02 est nouvelle");
    assert_eq!(
        report.ignored(),
        1,
        "2026-05-01 est un doublon (UNIQUE date,source)"
    );
}

#[tokio::test]
async fn inner_propagates_sql_error_instead_of_swallowing() {
    // Pas de bootstrap : la table body_measures_legacy n'existe pas -> l'INSERT
    // doit echouer et l'erreur remonter (au lieu d'etre avalee par `let _ =`).
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let json = r#"{
        "body_measures": [
            {"measured_at": "2026-05-01T08:00:00Z", "source": "manual", "weight": 70.0}
        ]
    }"#;
    let file = write_temp_json(json);

    let err = import_json_inner(&pool, file.path().to_str().unwrap())
        .await
        .expect_err("l'erreur SQL ne doit pas etre avalee");

    assert_eq!(
        err.kind(),
        "Db",
        "la variante Db doit remonter, recu: {err}"
    );
}

#[tokio::test]
async fn inner_is_atomic_and_rolls_back_on_error() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    // body_measures est traite en premier (valide). On supprime sleep_records pour
    // injecter une vraie erreur SQL ("no such table") APRES cette insertion.
    // NB : une violation de contrainte (CHECK/UNIQUE/NOT NULL) ne suffirait pas a
    // declencher le rollback car `INSERT OR IGNORE` l'avale par conception ; seule
    // une erreur structurelle (table/colonne absente) remonte via `?`.
    sqlx::query("DROP TABLE sleep_records")
        .execute(&pool)
        .await
        .unwrap();

    let json = r#"{
        "body_measures": [
            {"measured_at": "2026-05-01T08:00:00Z", "source": "manual", "weight": 70.0}
        ],
        "sleep_records": [
            {"date": "2026-05-01", "source": "manual", "sleep_score": 50}
        ]
    }"#;
    let file = write_temp_json(json);

    let result = import_json_inner(&pool, file.path().to_str().unwrap()).await;
    assert!(
        result.is_err(),
        "une table absente doit faire echouer l'import"
    );

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM body_measures_legacy")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        count.0, 0,
        "rollback : la ligne body_measures inseree avant l'erreur doit etre annulee"
    );
}

#[tokio::test]
async fn inner_persists_committed_rows_on_success() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let json = r#"{
        "body_measures": [
            {"measured_at": "2026-05-01T08:00:00Z", "source": "manual", "weight": 70.0},
            {"measured_at": "2026-05-02T08:00:00Z", "source": "manual", "weight": 71.0}
        ]
    }"#;
    let file = write_temp_json(json);

    let report = import_json_inner(&pool, file.path().to_str().unwrap())
        .await
        .expect("import doit reussir");

    assert_eq!(report.inserted(), 2);
    assert_eq!(report.ignored(), 0);

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM body_measures_legacy")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 2, "tx.commit doit persister les 2 lignes");
}

#[tokio::test]
async fn inner_returns_zero_on_empty_json() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let file = write_temp_json("{}");

    let report = import_json_inner(&pool, file.path().to_str().unwrap())
        .await
        .expect("un JSON vide est valide");

    assert_eq!(report.inserted(), 0);
    assert_eq!(report.ignored(), 0);
}

#[tokio::test]
async fn inner_skips_constraint_violating_rows_without_error() {
    // INSERT OR IGNORE avale les violations de contrainte (ici une source
    // inconnue de la table sources, que la cle etrangere refuse) : la ligne est ignoree (rows_affected == 0),
    // pas une erreur. Elle est donc comptee en `ignored` et l'import reste Ok.
    // Ce test documente que le fail-fast ne se declenche QUE sur erreur structurelle.
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();

    let json = r#"{
        "blood_pressure": [
            {"measured_at": "2026-05-01T08:00:00Z", "source": "BAD_SOURCE", "systolic": 120, "diastolic": 80}
        ]
    }"#;
    let file = write_temp_json(json);

    let report = import_json_inner(&pool, file.path().to_str().unwrap())
        .await
        .expect("OR IGNORE ne doit pas faire echouer sur une violation de contrainte");

    assert_eq!(report.inserted(), 0, "la ligne a une source inconnue");
    assert_eq!(report.ignored(), 1, "comptee comme ignoree (OR IGNORE)");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM blood_pressure")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 0, "rien ne doit etre persiste");
}
