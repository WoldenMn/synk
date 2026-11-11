use sqlx::SqlitePool;
use synk_lib::services::sync_log::{purge as purge_sync_log, record_sync, SyncLogRecord};

async fn insert_sync(pool: &SqlitePool, data_type: &str, started_modifier: &str, status: &str) {
    sqlx::query(
        "INSERT INTO sync_log (started_at, finished_at, data_type, duration_ms, records_added, status) \
         VALUES (datetime('now', ?1), datetime('now', ?1), ?2, 1, 0, ?3)",
    )
    .bind(started_modifier)
    .bind(data_type)
    .bind(status)
    .execute(pool)
    .await
    .expect("ligne");
}

#[tokio::test]
async fn sync_log_keeps_a_thousand_recent_rows_and_the_last_success_of_each_type() {
    let pool = crate::common::memory_pool().await;
    let mut tx = pool.begin().await.expect("transaction");
    for i in 0..1_200 {
        sqlx::query(
            "INSERT INTO sync_log (started_at, finished_at, data_type, duration_ms, records_added, status) \
             VALUES (datetime('now', ?1), datetime('now', ?1), 'sleep', 1, 0, 'success')",
        )
        .bind(format!("-{i} minutes"))
        .execute(&mut *tx)
        .await
        .expect("ligne");
    }
    tx.commit().await.expect("validation");
    insert_sync(&pool, "sleep", "-100 days", "error").await;
    insert_sync(&pool, "activity", "-100 days", "success").await;

    let removed = purge_sync_log(&pool, 1_000, 90).await.expect("purge");

    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_log")
        .fetch_one(&pool)
        .await
        .expect("compte");
    assert_eq!(left, 1_001);
    assert_eq!(removed, 201);
    let activity: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sync_log WHERE data_type = 'activity'")
            .fetch_one(&pool)
            .await
            .expect("compte");
    assert_eq!(activity, 1, "la derniere synchro reussie d'un type reste");
}

#[tokio::test]
async fn sync_log_purge_by_age_removes_old_rows_below_the_cap() {
    let pool = crate::common::memory_pool().await;
    insert_sync(&pool, "sleep", "-1 days", "success").await;
    insert_sync(&pool, "sleep", "-100 days", "error").await;
    insert_sync(&pool, "sleep", "-95 days", "success").await;

    let removed = purge_sync_log(&pool, 1_000, 90).await.expect("purge");

    assert_eq!(removed, 2);
}

#[tokio::test]
async fn a_recorded_sync_error_is_truncated_to_2000_characters() {
    let pool = crate::common::memory_pool().await;
    let long = "e".repeat(5_000);

    record_sync(
        &pool,
        &SyncLogRecord {
            source: "withings",
            data_type: "sleep",
            started_at: "2026-10-25T08:00:00Z",
            duration_ms: 10,
            records_added: 0,
            status: "error",
            error: Some(&long),
        },
    )
    .await;

    let stored: i64 = sqlx::query_scalar("SELECT LENGTH(error) FROM sync_log")
        .fetch_one(&pool)
        .await
        .expect("longueur");
    assert_eq!(stored, 2_000);
}

#[tokio::test]
async fn optimize_on_open_gathers_statistics_for_filled_tables() {
    let pool = crate::common::memory_pool().await;
    let mut tx = pool.begin().await.expect("transaction");
    for i in 0..300 {
        sqlx::query("INSERT INTO sleep_records (date, source, sleep_score) VALUES (date('2026-01-01', ?1), 'manual', 80)")
            .bind(format!("+{i} days"))
            .execute(&mut *tx)
            .await
            .expect("sommeil");
    }
    tx.commit().await.expect("validation");

    synk_lib::db::optimize_on_open(&pool)
        .await
        .expect("optimize");

    let stats: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_stat1 WHERE tbl = 'sleep_records'")
            .fetch_one(&pool)
            .await
            .expect("statistiques");
    assert!(stats > 0);
    synk_lib::db::optimize(&pool).await.expect("optimize");
}

#[tokio::test]
async fn detectors_use_an_index() {
    use synk_lib::metric_repository::{series_sql, LegacyTable};
    use synk_lib::services::insights::detectors::{heart_rate_anomaly, hypertension};
    let pool = crate::common::memory_pool().await;
    let weight_series = series_sql(LegacyTable::BodyMeasures, "weight").expect("requete du poids");
    let steps_series = series_sql(LegacyTable::Activity, "steps").expect("requete des pas");
    for (name, sql) in [
        ("hypertension", hypertension::SQL),
        ("poids (weight_trend)", weight_series.as_str()),
        ("pas (activity_delta)", steps_series.as_str()),
        ("heart_rate_anomaly", heart_rate_anomaly::SQL),
    ] {
        let plan: Vec<(i64, i64, i64, String)> =
            sqlx::query_as(sqlx::AssertSqlSafe(format!("EXPLAIN QUERY PLAN {sql}")))
                .fetch_all(&pool)
                .await
                .expect(name);
        assert!(!plan.is_empty(), "{name}");
        assert!(
            plan.iter().all(|(_, _, _, d)| !d.starts_with("SCAN")),
            "{name} : {plan:?}"
        );
    }
}
