use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use synk_lib::commands::crud::{select_range, CrudTable};

#[derive(Debug, sqlx::FromRow, PartialEq)]
struct TestRow {
    id: i64,
    measured_at: String,
    source: String,
    value: i64,
}

impl CrudTable for TestRow {
    const TABLE: &'static str = "crud_test";
    const SELECT_COLS: &'static str = "id, measured_at, source, value";
    const DATE_COL: &'static str = "measured_at";
}

async fn setup_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE crud_test (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            measured_at TEXT NOT NULL,
            source TEXT NOT NULL,
            value INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

async fn insert_row(pool: &SqlitePool, measured_at: &str, source: &str, value: i64) {
    sqlx::query("INSERT INTO crud_test (measured_at, source, value) VALUES (?1, ?2, ?3)")
        .bind(measured_at)
        .bind(source)
        .bind(value)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_select_range_returns_rows_in_asc_order() {
    let pool = setup_pool().await;
    insert_row(&pool, "2026-04-20", "manual", 1).await;
    insert_row(&pool, "2026-04-10", "manual", 2).await;
    insert_row(&pool, "2026-04-15", "manual", 3).await;

    let rows: Vec<TestRow> = select_range(&pool, "2026-04-01".into(), "2026-04-30".into())
        .await
        .unwrap();

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].measured_at, "2026-04-10");
    assert_eq!(rows[1].measured_at, "2026-04-15");
    assert_eq!(rows[2].measured_at, "2026-04-20");
}

#[derive(Debug, sqlx::FromRow, PartialEq)]
struct TestRowDesc {
    id: i64,
    measured_at: String,
    source: String,
    value: i64,
}

impl CrudTable for TestRowDesc {
    const TABLE: &'static str = "crud_test";
    const SELECT_COLS: &'static str = "id, measured_at, source, value";
    const DATE_COL: &'static str = "measured_at";
    const RANGE_ORDER: &'static str = "DESC";
}

#[tokio::test]
async fn test_select_range_returns_rows_in_desc_order_when_overridden() {
    let pool = setup_pool().await;
    insert_row(&pool, "2026-04-20", "manual", 1).await;
    insert_row(&pool, "2026-04-10", "manual", 2).await;
    insert_row(&pool, "2026-04-15", "manual", 3).await;

    let rows: Vec<TestRowDesc> = select_range(&pool, "2026-04-01".into(), "2026-04-30".into())
        .await
        .unwrap();

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].measured_at, "2026-04-20");
    assert_eq!(rows[1].measured_at, "2026-04-15");
    assert_eq!(rows[2].measured_at, "2026-04-10");
}

#[tokio::test]
async fn test_select_range_includes_timestamps_of_the_last_day() {
    let pool = setup_pool().await;
    insert_row(&pool, "2026-09-12T21:59:00Z", "withings", 1).await;
    insert_row(&pool, "2026-09-13T06:30:00Z", "withings", 2).await;
    insert_row(&pool, "2026-09-13T23:10:00+02:00", "manual", 3).await;
    insert_row(&pool, "2026-09-14T00:05:00Z", "withings", 4).await;

    let rows: Vec<TestRow> = select_range(
        &pool,
        "2026-09-12T00:00:00+02:00".into(),
        "2026-09-13T13:05:00+02:00".into(),
    )
    .await
    .unwrap();

    let values: Vec<i64> = rows.iter().map(|r| r.value).collect();
    assert_eq!(
        values,
        vec![1, 2, 3],
        "le dernier jour doit etre inclus, le lendemain exclu"
    );
}

#[tokio::test]
async fn test_select_range_rejects_an_unparsable_upper_bound() {
    let pool = setup_pool().await;

    let res: Result<Vec<TestRow>, _> =
        select_range(&pool, "2026-09-01".into(), "pas-une-date".into()).await;

    assert!(
        res.is_err(),
        "une borne illisible doit remonter une erreur, pas une liste vide"
    );
}

#[tokio::test]
async fn test_select_latest_returns_most_recent_row() {
    let pool = setup_pool().await;
    insert_row(&pool, "2026-04-10", "manual", 1).await;
    insert_row(&pool, "2026-04-25", "manual", 2).await;
    insert_row(&pool, "2026-04-15", "manual", 3).await;

    let row: Option<TestRow> = synk_lib::commands::crud::select_latest(&pool)
        .await
        .unwrap();

    assert!(row.is_some());
    let row = row.unwrap();
    assert_eq!(row.measured_at, "2026-04-25");
    assert_eq!(row.value, 2);
}

#[tokio::test]
async fn test_select_latest_returns_none_when_empty() {
    let pool = setup_pool().await;

    let row: Option<TestRow> = synk_lib::commands::crud::select_latest(&pool)
        .await
        .unwrap();

    assert!(row.is_none());
}

async fn setup_pool_with_audit() -> SqlitePool {
    let pool = setup_pool().await;
    sqlx::query(
        "CREATE TABLE audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            action TEXT NOT NULL,
            entity TEXT NOT NULL,
            entity_id INTEGER,
            details TEXT,
            occurred_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn test_delete_with_audit_removes_row_and_logs() {
    let pool = setup_pool_with_audit().await;
    insert_row(&pool, "2026-04-20", "manual", 1).await;

    synk_lib::commands::crud::delete_with_audit(&pool, "crud_test", 1)
        .await
        .unwrap();

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM crud_test")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 0);

    let audit: (String, String, Option<i64>) =
        sqlx::query_as("SELECT action, entity, entity_id FROM audit_log ORDER BY id DESC LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(audit.0, "delete");
    assert_eq!(audit.1, "crud_test");
    assert_eq!(audit.2, Some(1));
}

#[tokio::test]
async fn test_delete_with_audit_refuses_a_synced_row() {
    let pool = setup_pool_with_audit().await;
    insert_row(&pool, "2026-04-20", "withings", 1).await;

    let err = synk_lib::commands::crud::delete_with_audit(&pool, "crud_test", 1)
        .await
        .unwrap_err();

    assert!(
        matches!(err, synk_lib::error::AppError::Forbidden(_)),
        "{err}"
    );
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM crud_test")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 1, "la ligne synchronisee reste en place");
    let audits: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        audits, 0,
        "aucune trace pour une suppression qui n'a pas eu lieu"
    );
}

#[tokio::test]
async fn test_delete_with_audit_on_a_missing_id_is_not_found() {
    let pool = setup_pool_with_audit().await;

    let err = synk_lib::commands::crud::delete_with_audit(&pool, "crud_test", 42)
        .await
        .unwrap_err();

    assert!(
        matches!(
            err,
            synk_lib::error::AppError::NotFound {
                entity: "crud_test",
                id: 42
            }
        ),
        "{err}"
    );
    let audits: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(audits, 0);
}

#[tokio::test]
async fn test_record_audit_insert_logs_action() {
    let pool = setup_pool_with_audit().await;

    synk_lib::commands::crud::record_audit_insert(&pool, "crud_test", 42).await;

    let audit: (String, String, Option<i64>) =
        sqlx::query_as("SELECT action, entity, entity_id FROM audit_log ORDER BY id DESC LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(audit.0, "insert");
    assert_eq!(audit.1, "crud_test");
    assert_eq!(audit.2, Some(42));
}

#[tokio::test]
async fn test_delete_by_id_removes_once_then_reports_not_found() {
    let pool = setup_pool_with_audit().await;
    insert_row(&pool, "2026-04-20", "withings", 1).await;

    synk_lib::commands::crud::delete_by_id(&pool, "crud_test", "crud_test", 1)
        .await
        .unwrap();
    let again = synk_lib::commands::crud::delete_by_id(&pool, "crud_test", "crud_test", 1).await;

    assert!(
        matches!(again, Err(synk_lib::error::AppError::NotFound { .. })),
        "{again:?}"
    );
    let audits: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(audits, 1, "une seule suppression reelle, une seule trace");
}
