use sqlx::SqlitePool;
use synk_lib::services::retention::run;

async fn count(pool: &SqlitePool, table: &str) -> i64 {
    let sql = match table {
        "error_log" => "SELECT COUNT(*) FROM error_log",
        "audit_log" => "SELECT COUNT(*) FROM audit_log",
        _ => "SELECT COUNT(*) FROM sync_log",
    };
    sqlx::query_scalar(sql)
        .fetch_one(pool)
        .await
        .expect("compte")
}

async fn fill_error_log(pool: &SqlitePool, rows: i64) {
    let mut tx = pool.begin().await.expect("transaction");
    for i in 0..rows {
        sqlx::query("INSERT INTO error_log (source, code, message) VALUES ('t', 'C', ?1)")
            .bind(format!("m{i}"))
            .execute(&mut *tx)
            .await
            .expect("ligne");
    }
    tx.commit().await.expect("validation");
}

#[tokio::test]
async fn the_error_log_is_capped_even_when_audit_purge_is_disabled() {
    let pool = crate::common::memory_pool().await;
    sqlx::query("UPDATE settings SET audit_auto_purge_enabled = 0 WHERE id = 1")
        .execute(&pool)
        .await
        .expect("reglage");
    fill_error_log(&pool, 2_050).await;

    let report = run(&pool, "2026-10-25 12:00:00").await;

    assert_eq!(count(&pool, "error_log").await, 2_000);
    assert_eq!(report.audit_removed, None);
}

#[tokio::test]
async fn an_audit_log_older_than_the_retention_is_purged() {
    let pool = crate::common::memory_pool().await;
    sqlx::query("INSERT INTO audit_log (action, entity, created_at) VALUES ('insert', 'x', '2020-01-01 00:00:00')")
        .execute(&pool)
        .await
        .expect("ligne");

    let report = run(&pool, "2026-10-25 12:00:00").await;

    assert_eq!(report.audit_removed, Some(1));
    assert_eq!(
        count(&pool, "audit_log").await,
        1,
        "la purge s'inscrit au journal"
    );
}

#[tokio::test]
async fn nothing_is_removed_from_a_young_database() {
    let pool = crate::common::memory_pool().await;

    let report = run(&pool, "2026-10-25 12:00:00").await;

    assert_eq!(report.sync_log_removed, 0);
    assert_eq!(report.audit_removed, None);
    assert_eq!(count(&pool, "sync_log").await, 0);
}
