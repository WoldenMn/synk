use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use synk_lib::db::bootstrap_schema;
use synk_lib::services::error_log::{
    auto_purge, get_error_log, log_error, purge_error_log, ErrorLogEntry,
};

async fn setup_pool() -> SqlitePool {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();
    bootstrap_schema(&p).await.unwrap();
    p
}

#[tokio::test]
async fn error_log_table_exists() {
    let pool = setup_pool().await;
    let name: Option<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='error_log'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert_eq!(name.as_deref(), Some("error_log"));
}

#[tokio::test]
async fn log_and_get_error() {
    let p = setup_pool().await;
    log_error(
        &p,
        "sync",
        "AUTH",
        "Session expiree",
        Some("withings_sync_all"),
    )
    .await
    .unwrap();
    log_error(&p, "mutation", "DATABASE", "locked", None)
        .await
        .unwrap();
    let all: Vec<ErrorLogEntry> = get_error_log(&p, 50, None).await.unwrap();
    assert_eq!(all.len(), 2);
    let auth_only = get_error_log(&p, 50, Some("AUTH")).await.unwrap();
    assert_eq!(auth_only.len(), 1);
    assert_eq!(auth_only[0].code, "AUTH");
    assert_eq!(auth_only[0].context.as_deref(), Some("withings_sync_all"));
}

#[tokio::test]
async fn purge_clears_all() {
    let p = setup_pool().await;
    log_error(&p, "manual", "VALIDATION", "x", None)
        .await
        .unwrap();
    purge_error_log(&p).await.unwrap();
    assert!(get_error_log(&p, 50, None).await.unwrap().is_empty());
}

#[tokio::test]
async fn auto_purge_keeps_recent_cap() {
    let p = setup_pool().await;
    for i in 0..2050 {
        log_error(&p, "sync", "NETWORK", &format!("e{i}"), None)
            .await
            .unwrap();
    }
    auto_purge(&p, 2000).await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM error_log")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(count, 2000);
}

// B-4 : message tronque a 2000 chars, context a 5000 chars
#[tokio::test]
async fn log_error_truncates_oversized_message() {
    let p = setup_pool().await;
    let long_msg: String = "x".repeat(5000);
    let long_ctx: String = "y".repeat(10000);
    log_error(&p, "sync", "NETWORK", &long_msg, Some(&long_ctx))
        .await
        .unwrap();
    let entry = get_error_log(&p, 1, None)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(
        entry.message.chars().count(),
        2000,
        "message doit etre tronque a 2000 chars"
    );
    assert_eq!(
        entry.context.unwrap().chars().count(),
        5000,
        "context doit etre tronque a 5000 chars"
    );
}
