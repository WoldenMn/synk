use sqlx::SqlitePool;
use synk_lib::services::sync_log::{purge, recent, record_sync, SyncLogRecord};

async fn insert(pool: &SqlitePool, source: &str, data_type: &str, age: &str, status: &str) {
    sqlx::query(
        "INSERT INTO sync_log (started_at, finished_at, data_type, duration_ms, records_added, status, source) \
         VALUES (datetime('now', ?1), datetime('now', ?1), ?2, 1, 0, ?3, ?4)",
    )
    .bind(age)
    .bind(data_type)
    .bind(status)
    .bind(source)
    .execute(pool)
    .await
    .expect("ligne");
}

#[tokio::test]
async fn a_row_written_without_a_source_belongs_to_withings() {
    let pool = crate::common::memory_pool().await;
    sqlx::query(
        "INSERT INTO sync_log (started_at, data_type, status) VALUES ('2026-10-25T08:00:00Z', 'sleep', 'success')",
    )
    .execute(&pool)
    .await
    .expect("ligne sans source");

    let rows = recent(&pool, Some(10)).await.expect("lecture");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source, "withings");
}

#[tokio::test]
async fn a_synchronisation_is_logged_under_its_source() {
    let pool = crate::common::memory_pool().await;

    record_sync(
        &pool,
        &SyncLogRecord {
            source: "oura",
            data_type: "sleep",
            started_at: "2026-10-25T08:00:00Z",
            duration_ms: 12,
            records_added: 3,
            status: "success",
            error: None,
        },
    )
    .await;

    let rows = recent(&pool, Some(10)).await.expect("lecture");
    assert_eq!(rows[0].source, "oura");
    assert_eq!(rows[0].records_added, Some(3));
}

#[tokio::test]
async fn the_purge_keeps_the_last_success_of_each_source_and_type() {
    let pool = crate::common::memory_pool().await;
    insert(&pool, "withings", "sleep", "-100 days", "success").await;
    insert(&pool, "oura", "sleep", "-101 days", "success").await;
    insert(&pool, "oura", "sleep", "-102 days", "success").await;

    let removed = purge(&pool, 1_000, 90).await.expect("purge");

    assert_eq!(removed, 1, "seul l'ancien doublon d'oura part");
    let left: Vec<(String,)> = sqlx::query_as("SELECT source FROM sync_log ORDER BY source")
        .fetch_all(&pool)
        .await
        .expect("lignes");
    assert_eq!(
        left.into_iter().map(|r| r.0).collect::<Vec<_>>(),
        vec!["oura".to_string(), "withings".to_string()]
    );
}
