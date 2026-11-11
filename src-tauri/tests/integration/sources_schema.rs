use std::collections::BTreeMap;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};
use synk_lib::db::{bootstrap_schema, bootstrap_schema_up_to, LATEST_VERSION};

use crate::common::test_vault;

const REBUILT: [&str; 8] = [
    "body_measures_legacy",
    "vitals_legacy",
    "blood_pressure",
    "sleep_records",
    "activity",
    "workouts",
    "heart_records",
    "metrics",
];

const SEEDED: [&str; 12] = [
    "withings",
    "manual",
    "derived",
    "import",
    "oura",
    "polar",
    "whoop",
    "google_health",
    "apple_health",
    "fit",
    "trace",
    "csv",
];

async fn pool() -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("PRAGMA foreign_keys=ON")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect(":memory:")
        .await
        .unwrap()
}

async fn q(pool: &SqlitePool, sql: String) -> Vec<sqlx::sqlite::SqliteRow> {
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn columns(pool: &SqlitePool, table: &str) -> Vec<(String, String)> {
    q(pool, format!("PRAGMA table_info(\"{table}\")"))
        .await
        .iter()
        .map(|r| (r.get::<String, _>("name"), r.get::<String, _>("type")))
        .collect()
}

fn fill(table: &str, column: &str, ty: &str, n: i64, source: &str) -> String {
    match column {
        "source" => format!("'{source}'"),
        "vital_type" => "'spo2'".to_string(),
        "measured_at" | "created_at" | "updated_at" | "start_at" | "end_at" | "timestamp" => {
            format!("'2026-01-{:02}T08:00:00Z'", n + 1)
        }
        "date" => format!("'2026-01-{:02}'", n + 1),
        "definition_id" => "1".to_string(),
        _ if ty.starts_with("INT") => format!("{}", n * 10 + 7),
        _ if ty == "REAL" => format!("{}.5", n * 10 + 3),
        _ => format!("'{table}-{column}-{n}'"),
    }
}

async fn fill_table(pool: &SqlitePool, table: &str, rows: i64, source: &str) {
    let cols: Vec<(String, String)> = columns(pool, table)
        .await
        .into_iter()
        .filter(|(c, _)| c != "id")
        .collect();
    let names: Vec<&str> = cols.iter().map(|(c, _)| c.as_str()).collect();
    for n in 0..rows {
        let values: Vec<String> = cols
            .iter()
            .map(|(c, t)| fill(table, c, t, n, source))
            .collect();
        let sql = format!(
            "INSERT INTO {table} ({}) VALUES ({})",
            names.join(", "),
            values.join(", ")
        );
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("{table}: {e}"));
    }
}

async fn v18_with_data() -> SqlitePool {
    let pool = pool().await;
    bootstrap_schema_up_to(&pool, 18).await.unwrap();
    for t in REBUILT {
        fill_table(&pool, t, 3, "withings").await;
    }
    for t in [
        "blood_pressure",
        "sleep_records",
        "activity",
        "body_measures_legacy",
    ] {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE {t} SET source = 'manual' WHERE id = 2"
        )))
        .execute(&pool)
        .await
        .unwrap();
    }
    sqlx::query(
        "INSERT INTO withings_auth (id, access_token, refresh_token, expires_at, token_type, withings_user_id) \
         VALUES (1, 'enc:v1:AAAA', 'enc:v1:BBBB', 1790000000, 'Bearer', 'u-42')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE withings_credentials SET client_id = 'enc:v1:CCCC', client_secret = 'enc:v1:DDDD'",
    )
    .execute(&pool)
    .await
    .ok();
    sqlx::query("INSERT OR IGNORE INTO withings_credentials (id, client_id, client_secret) VALUES (1, 'enc:v1:CCCC', 'enc:v1:DDDD')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE sync_state SET last_sync_at = id * 100")
        .execute(&pool)
        .await
        .unwrap();
    pool
}

async fn dump(pool: &SqlitePool, table: &str, cols: &[String]) -> Vec<Vec<String>> {
    let list = cols
        .iter()
        .map(|c| format!("CAST(\"{c}\" AS TEXT)"))
        .collect::<Vec<_>>()
        .join(", ");
    q(pool, format!("SELECT {list} FROM {table} ORDER BY 1"))
        .await
        .iter()
        .map(|r| {
            (0..cols.len())
                .map(|i| r.get::<Option<String>, _>(i).unwrap_or("<NULL>".into()))
                .collect()
        })
        .collect()
}

async fn snapshot(
    pool: &SqlitePool,
    tables: &[&str],
) -> BTreeMap<String, (Vec<String>, Vec<Vec<String>>)> {
    let mut out = BTreeMap::new();
    for t in tables {
        let cols: Vec<String> = columns(pool, t).await.into_iter().map(|(c, _)| c).collect();
        let rows = dump(pool, t, &cols).await;
        out.insert((*t).to_string(), (cols, rows));
    }
    out
}

async fn fk_violations(pool: &SqlitePool) -> usize {
    q(pool, "PRAGMA foreign_key_check".to_string()).await.len()
}

async fn integrity(pool: &SqlitePool) -> String {
    sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_new_base_seeds_the_sources_table() {
    let pool = pool().await;
    bootstrap_schema(&pool).await.unwrap();
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM sources ORDER BY sort_order")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(ids, SEEDED);
}

#[tokio::test]
async fn every_registered_source_is_in_the_sources_table() {
    let pool = pool().await;
    bootstrap_schema(&pool).await.unwrap();
    for source in synk_lib::sources::all_sources() {
        let found: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sources WHERE id = ?1")
            .bind(source.id().as_str())
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(found, 1, "{}", source.id().as_str());
    }
}

#[tokio::test]
async fn the_source_columns_reference_sources_instead_of_a_check() {
    let pool = pool().await;
    bootstrap_schema(&pool).await.unwrap();
    for t in REBUILT {
        let fks = q(&pool, format!("PRAGMA foreign_key_list(\"{t}\")")).await;
        assert!(
            fks.iter().any(|r| r.get::<String, _>("table") == "sources"
                && r.get::<String, _>("from") == "source"),
            "{t} sans cle etrangere sur sources"
        );
        let ddl: String = sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE name = ?1")
            .bind(t)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(!ddl.contains("source IN"), "{t} garde un CHECK : {ddl}");
        let names: Vec<String> = columns(&pool, t).await.into_iter().map(|c| c.0).collect();
        assert!(names.contains(&"external_id".to_string()), "{t}");
    }
}

#[tokio::test]
async fn a_new_source_is_accepted_and_an_unknown_one_refused() {
    let pool = pool().await;
    bootstrap_schema(&pool).await.unwrap();
    sqlx::query("INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES ('2026-02-01T08:00:00Z', 'oura', 70)")
        .execute(&pool)
        .await
        .unwrap();
    let refused = sqlx::query("INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES ('2026-02-01T08:00:00Z', 'inconnue', 70)")
        .execute(&pool)
        .await;
    assert!(refused.is_err());
}

#[tokio::test]
async fn external_id_is_unique_per_source_and_free_across_sources() {
    let pool = pool().await;
    bootstrap_schema(&pool).await.unwrap();
    let insert = |source: &'static str| {
        let pool = pool.clone();
        async move {
            sqlx::query("INSERT INTO body_measures_legacy (measured_at, source, weight, external_id) VALUES ('2026-02-01T08:00:00Z', ?1, 70, 'abc')")
                .bind(source)
                .execute(&pool)
                .await
        }
    };
    insert("csv").await.unwrap();
    insert("oura").await.unwrap();
    assert!(insert("csv").await.is_err());
}

#[tokio::test]
async fn the_withings_tables_are_views_over_the_source_tables() {
    let pool = pool().await;
    bootstrap_schema(&pool).await.unwrap();
    for v in ["withings_auth", "withings_credentials", "sync_state"] {
        let kind: String = sqlx::query_scalar("SELECT type FROM sqlite_master WHERE name = ?1")
            .bind(v)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(kind, "view", "{v}");
    }
    sqlx::query("INSERT INTO source_auth (source_id, access_token, refresh_token, expires_at) VALUES ('withings', 'a', 'b', 9)")
        .execute(&pool)
        .await
        .unwrap();
    let (id, token): (i64, String) = sqlx::query_as("SELECT id, refresh_token FROM withings_auth")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!((id, token.as_str()), (1, "b"));
}

#[tokio::test]
async fn migrating_a_v18_base_keeps_every_row_and_every_secret() {
    let pool = v18_with_data().await;
    let tables: Vec<&str> = REBUILT
        .iter()
        .copied()
        .chain(["withings_auth", "withings_credentials", "sync_state"])
        .collect();
    let before = snapshot(&pool, &tables).await;
    assert!(before["withings_auth"].1.len() == 1 && before["sync_state"].1.len() == 7);
    assert!(REBUILT.iter().all(|t| before[*t].1.len() == 3));

    bootstrap_schema(&pool).await.unwrap();

    let version: i64 = sqlx::query_scalar("SELECT MAX(version) FROM schema_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(version, LATEST_VERSION);
    assert_eq!(integrity(&pool).await, "ok");
    assert_eq!(fk_violations(&pool).await, 0);
    for t in &tables {
        let (old_cols, old_rows) = &before[*t];
        let after = dump(&pool, t, old_cols).await;
        assert_eq!(&after, old_rows, "{t} a change");
    }
    let secret: (String, String) = sqlx::query_as(
        "SELECT access_token, refresh_token FROM source_auth WHERE source_id = 'withings'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(secret, ("enc:v1:AAAA".into(), "enc:v1:BBBB".into()));
}

#[tokio::test]
async fn autoincrement_sequences_survive_the_rebuild() {
    let pool = pool().await;
    bootstrap_schema_up_to(&pool, 18).await.unwrap();
    fill_table(&pool, "activity", 3, "withings").await;
    sqlx::query("DELETE FROM activity WHERE id = 3")
        .execute(&pool)
        .await
        .unwrap();
    bootstrap_schema(&pool).await.unwrap();
    sqlx::query("INSERT INTO activity (date, source, steps) VALUES ('2027-01-01', 'withings', 1)")
        .execute(&pool)
        .await
        .unwrap();
    let id: i64 = sqlx::query_scalar("SELECT id FROM activity WHERE date = '2027-01-01'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(id, 4);
}

#[tokio::test]
async fn migrating_an_up_to_date_base_changes_nothing() {
    let pool = pool().await;
    bootstrap_schema(&pool).await.unwrap();
    let tables: Vec<&str> = REBUILT
        .iter()
        .copied()
        .chain(["sources", "source_state"])
        .collect();
    for t in REBUILT {
        fill_table(&pool, t, 2, "manual").await;
    }
    let before = snapshot(&pool, &tables).await;
    let schema_before: Vec<String> =
        sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
            .fetch_all(&pool)
            .await
            .unwrap();

    bootstrap_schema(&pool).await.unwrap();

    assert_eq!(snapshot(&pool, &tables).await, before);
    let schema_after: Vec<String> =
        sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(schema_after, schema_before);
    assert_eq!(integrity(&pool).await, "ok");
    assert_eq!(fk_violations(&pool).await, 0);
}

#[tokio::test]
async fn an_empty_base_migrates_to_the_latest_version() {
    let pool = pool().await;
    bootstrap_schema(&pool).await.unwrap();
    let max: i64 = sqlx::query_scalar("SELECT MAX(version) FROM schema_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(max, LATEST_VERSION);
    assert_eq!(integrity(&pool).await, "ok");
    assert_eq!(fk_violations(&pool).await, 0);
    for t in REBUILT {
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM {t}")))
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(n, 0, "{t}");
    }
}

#[tokio::test]
async fn an_unknown_legacy_source_is_registered_not_lost() {
    let pool = pool().await;
    bootstrap_schema_up_to(&pool, 18).await.unwrap();
    sqlx::query("PRAGMA ignore_check_constraints=ON")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO heart_records (timestamp, source, heart_rate) VALUES ('2026-01-01T00:00:00Z', 'garmin', 60)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("PRAGMA ignore_check_constraints=OFF")
        .execute(&pool)
        .await
        .unwrap();
    bootstrap_schema(&pool).await.unwrap();
    let source: String = sqlx::query_scalar("SELECT source FROM heart_records")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(source, "garmin");
    let label: String = sqlx::query_scalar("SELECT label FROM sources WHERE id = 'garmin'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(label, "garmin");
    assert_eq!(fk_violations(&pool).await, 0);
}

#[tokio::test]
async fn copied_tokens_still_open_with_the_same_key() {
    let pool = pool().await;
    bootstrap_schema_up_to(&pool, 18).await.unwrap();
    let vault = test_vault();
    let access = vault.seal_str("jeton-acces").unwrap();
    let refresh = vault.seal_str("jeton-refresh").unwrap();
    sqlx::query("INSERT INTO withings_auth (id, access_token, refresh_token, expires_at) VALUES (1, ?1, ?2, 1)")
        .bind(&access)
        .bind(&refresh)
        .execute(&pool)
        .await
        .unwrap();
    bootstrap_schema(&pool).await.unwrap();
    let stored =
        synk_lib::sources::store::load_auth(&pool, &vault, synk_lib::sources::SourceId::WITHINGS)
            .await
            .unwrap()
            .unwrap();
    assert_eq!(stored.access_token.as_str(), "jeton-acces");
    assert_eq!(stored.refresh_token.as_str(), "jeton-refresh");
}
