use synk_lib::db::run_post_ready_seed;
use synk_lib::sources::withings::sync::{get_last_sync, update_last_sync};

#[tokio::test]
async fn source_state_accepts_the_metrics_cursor() {
    let pool = crate::common::memory_pool().await;
    // Avant migration 010, le CHECK rejetait 'metrics' (CHECK constraint failed).
    // Apres, la migration cree la ligne et un upsert de curseur (pattern exact de
    // update_last_sync) doit reussir.
    let res = sqlx::query(
        "INSERT INTO source_state (source_id, domain, last_sync_at, updated_at) VALUES ('withings', 'metrics', 123, datetime('now')) \
         ON CONFLICT(source_id, domain) DO UPDATE SET last_sync_at = excluded.last_sync_at",
    )
    .execute(&pool)
    .await;
    assert!(
        res.is_ok(),
        "source_state doit accepter un curseur 'metrics' apres migration 010: {res:?}"
    );
    let v: i64 =
        sqlx::query_scalar("SELECT last_sync_at FROM sync_state WHERE data_type='metrics'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(v, 123);
}

#[tokio::test]
async fn metrics_seeded_after_post_ready_seed() {
    let pool = crate::common::memory_pool().await;
    run_post_ready_seed(&pool).await;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sync_state WHERE data_type='metrics'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1, "'metrics' doit etre seedee dans sync_state");
}

#[tokio::test]
async fn update_last_sync_persists_explicit_cursor() {
    let pool = crate::common::memory_pool().await;
    update_last_sync(&pool, "heart", 999_999).await.unwrap();
    let got = get_last_sync(&pool, "heart").await.unwrap();
    assert_eq!(got, 999_999, "doit persister le curseur fourni, pas now()");
}

#[tokio::test]
async fn update_last_sync_works_for_metrics() {
    let pool = crate::common::memory_pool().await;
    update_last_sync(&pool, "metrics", 555).await.unwrap();
    assert_eq!(get_last_sync(&pool, "metrics").await.unwrap(), 555);
}

#[tokio::test]
async fn a_day_with_zero_steps_is_a_point_of_the_series() {
    let pool = crate::common::memory_pool().await;
    sqlx::query("INSERT INTO activity (date, source, steps) VALUES ('2026-01-01', 'withings', 0)")
        .execute(&pool)
        .await
        .unwrap();
    let points =
        synk_lib::metric_repository::read_metric_series(&pool, "steps", "2026-01-01", "2026-01-31")
            .await
            .unwrap();
    assert_eq!(points.len(), 1);
    assert_eq!(points[0].value, 0.0, "#15 : un jour a 0 pas reste un point");
}
