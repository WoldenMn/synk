use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

// Les dates absolues transforment un test en bombe a retardement : celui-ci
// echouait depuis que 2026-05-01 est sorti de la fenetre de 90 jours.
// Toute date de test est donc calculee par rapport a l'instant d'execution.
fn days_ago(days: i64) -> String {
    (chrono::Local::now() - chrono::Duration::days(days))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

async fn setup_audit_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            action TEXT NOT NULL,
            entity TEXT NOT NULL,
            entity_id INTEGER,
            details TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

async fn insert_audit(
    pool: &SqlitePool,
    action: &str,
    entity: &str,
    entity_id: Option<i64>,
    created_at: &str,
) {
    sqlx::query(
        "INSERT INTO audit_log (action, entity, entity_id, created_at) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(action)
    .bind(entity)
    .bind(entity_id)
    .bind(created_at)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn test_purge_removes_entries_older_than_days() {
    let pool = setup_audit_pool().await;
    insert_audit(&pool, "insert", "blood_pressure", Some(1), &days_ago(400)).await;
    insert_audit(&pool, "delete", "sleep_records", Some(2), &days_ago(10)).await;

    let deleted = synk_lib::services::audit::purge_audit_log(&pool, 90)
        .await
        .unwrap();

    assert_eq!(deleted, 1, "1 entree de 2025 supprimee");
    let remaining: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(remaining.0 >= 1, "entree recente conservee + log de purge");
}

#[tokio::test]
async fn test_purge_keeps_recent_entries() {
    let pool = setup_audit_pool().await;
    insert_audit(&pool, "insert", "x", Some(1), &days_ago(10)).await;

    let deleted = synk_lib::services::audit::purge_audit_log(&pool, 90)
        .await
        .unwrap();

    assert_eq!(deleted, 0);
}

#[tokio::test]
async fn test_get_audit_metrics_returns_accurate_counts() {
    let pool = setup_audit_pool().await;
    let oldest = days_ago(120);
    let middle = days_ago(60);
    let newest = days_ago(10);
    insert_audit(&pool, "insert", "x", Some(1), &oldest).await;
    insert_audit(&pool, "insert", "y", Some(2), &middle).await;
    insert_audit(&pool, "delete", "z", Some(3), &newest).await;

    let metrics = synk_lib::services::audit::get_audit_metrics(&pool)
        .await
        .unwrap();

    assert_eq!(metrics.total_count, 3);
    assert_eq!(metrics.insert_count, 2);
    assert_eq!(metrics.delete_count, 1);
    assert_eq!(metrics.oldest_at.unwrap(), oldest);
    assert_eq!(metrics.newest_at.unwrap(), newest);
}

// B-8 : borne haute older_than_days sur purge_audit_log_inner
#[tokio::test]
async fn test_purge_rejects_older_than_days_above_max() {
    let pool = setup_audit_pool().await;
    let result = synk_lib::services::audit::purge_audit_log(&pool, 999_999).await;
    assert!(result.is_err(), "older_than_days=999999 doit retourner Err");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("36500") || msg.contains("validation") || msg.contains("Validation"),
        "message d'erreur attendu: {msg}"
    );
}
