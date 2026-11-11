use sqlx::SqlitePool;
use synk_lib::db::bootstrap_schema;
use synk_lib::services::audit::record_audit;

async fn setup_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    bootstrap_schema(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn record_audit_inserts_entry() {
    let pool = setup_pool().await;

    let id = record_audit(&pool, "delete", "body_measures", Some(42), None)
        .await
        .unwrap();
    assert!(id > 0);

    let row: (String, String, Option<i64>) =
        sqlx::query_as("SELECT action, entity, entity_id FROM audit_log WHERE id = ?1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row.0, "delete");
    assert_eq!(row.1, "body_measures");
    assert_eq!(row.2, Some(42));
}

#[tokio::test]
async fn record_audit_with_details() {
    let pool = setup_pool().await;

    record_audit(&pool, "delete", "goals", None, Some("weight"))
        .await
        .unwrap();

    let row: (String, Option<i64>, Option<String>) = sqlx::query_as(
        "SELECT entity, entity_id, details FROM audit_log WHERE entity = 'goals' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "goals");
    assert_eq!(row.1, None);
    assert_eq!(row.2, Some("weight".to_string()));
}
