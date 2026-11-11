use sqlx::SqlitePool;
use synk_lib::services::sync_log::{clear_sync_log, last_successful_sync};

async fn bootstrapped_pool(dir: &tempfile::TempDir) -> SqlitePool {
    synk_lib::db::init_pool(dir.path().join("synk.db"))
        .await
        .expect("base temporaire initialisee")
}

async fn insert_log(pool: &SqlitePool, finished_at: &str, status: &str) {
    insert_typed_log(pool, "measures", finished_at, status).await;
}

async fn insert_typed_log(pool: &SqlitePool, data_type: &str, finished_at: &str, status: &str) {
    sqlx::query(
        "INSERT INTO sync_log (started_at, finished_at, data_type, duration_ms, records_added, status, error) \
         VALUES (?1, ?1, ?3, 10, 1, ?2, NULL)",
    )
    .bind(finished_at)
    .bind(status)
    .bind(data_type)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn empty_log_has_no_last_sync() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    assert_eq!(last_successful_sync(&pool).await.unwrap(), None);
}

#[tokio::test]
async fn returns_the_latest_success_as_utc_iso() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    insert_log(&pool, "2026-09-30 08:00:00", "success").await;
    insert_log(&pool, "2026-10-01 09:30:00", "success").await;
    insert_log(&pool, "2026-10-02 10:00:00", "error").await;
    assert_eq!(
        last_successful_sync(&pool).await.unwrap(),
        Some("2026-10-01T09:30:00Z".to_string())
    );
}

#[tokio::test]
async fn clearing_the_log_keeps_the_last_successful_sync() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    insert_log(&pool, "2026-09-30 08:00:00", "success").await;
    insert_log(&pool, "2026-10-01 09:30:00", "success").await;
    insert_log(&pool, "2026-10-02 10:00:00", "error").await;

    clear_sync_log(&pool).await.unwrap();

    assert_eq!(
        last_successful_sync(&pool).await.unwrap(),
        Some("2026-10-01T09:30:00Z".to_string())
    );
    let (remaining,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sync_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining, 1);
}

#[tokio::test]
async fn clearing_a_log_without_success_empties_it() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    insert_log(&pool, "2026-10-02 10:00:00", "error").await;

    clear_sync_log(&pool).await.unwrap();

    let (remaining,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sync_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining, 0);
}

#[tokio::test]
async fn a_partial_sync_does_not_move_the_last_success() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    insert_typed_log(&pool, "measures", "2026-10-01 09:30:00", "success").await;
    insert_typed_log(&pool, "sleep", "2026-10-01 09:31:00", "success").await;
    insert_typed_log(&pool, "measures", "2026-10-02 10:00:00", "error").await;
    insert_typed_log(&pool, "devices", "2026-10-02 10:00:05", "success").await;

    assert_eq!(
        last_successful_sync(&pool).await.unwrap(),
        Some("2026-10-01T09:30:00Z".to_string())
    );
}

#[tokio::test]
async fn a_module_that_never_succeeded_does_not_hold_the_date_back() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    insert_typed_log(&pool, "measures", "2026-10-01 09:30:00", "success").await;
    insert_typed_log(&pool, "heart", "2026-10-01 09:30:05", "error").await;

    assert_eq!(
        last_successful_sync(&pool).await.unwrap(),
        Some("2026-10-01T09:30:00Z".to_string())
    );
}

#[tokio::test]
async fn clearing_the_log_keeps_the_last_success_of_every_module() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    insert_typed_log(&pool, "measures", "2026-09-30 08:00:00", "success").await;
    insert_typed_log(&pool, "measures", "2026-10-01 09:30:00", "success").await;
    insert_typed_log(&pool, "sleep", "2026-10-01 09:31:00", "success").await;
    insert_typed_log(&pool, "sleep", "2026-10-02 10:00:00", "error").await;

    clear_sync_log(&pool).await.unwrap();

    let (remaining,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sync_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining, 2);
    assert_eq!(
        last_successful_sync(&pool).await.unwrap(),
        Some("2026-10-01T09:30:00Z".to_string())
    );
}
