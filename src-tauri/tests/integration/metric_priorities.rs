use sqlx::SqlitePool;
use synk_lib::services::metrics::{priorities, set_priority};

use crate::common::memory_pool;

async fn weigh(pool: &SqlitePool, source: &str, at: &str, kg: f64) {
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES (?1, ?2, ?3)",
    )
    .bind(at)
    .bind(source)
    .bind(kg)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn only_a_metric_with_several_sources_is_listed() {
    let pool = memory_pool().await;
    weigh(&pool, "withings", "2026-03-01T07:00:00Z", 80.0).await;
    weigh(&pool, "withings", "2026-03-02T07:00:00Z", 79.5).await;

    assert!(priorities(&pool).await.unwrap().is_empty());

    weigh(&pool, "manual", "2026-03-02T09:00:00Z", 79.8).await;
    let listed = priorities(&pool).await.unwrap();

    let weight = listed.iter().find(|m| m.slug == "weight").expect("poids");
    assert_eq!(weight.label, "Poids");
    assert_eq!(weight.sources.len(), 2);
    assert!(weight.sources.contains(&"withings".to_string()));
    assert!(weight.sources.contains(&"manual".to_string()));
    assert!(!weight.custom);
    assert!(listed.iter().all(|m| m.slug != "fat_ratio"));
}

#[tokio::test]
async fn the_chosen_order_comes_first_and_resetting_restores_the_automatic_one() {
    let pool = memory_pool().await;
    weigh(&pool, "withings", "2026-03-01T07:00:00Z", 80.0).await;
    weigh(&pool, "manual", "2026-03-02T09:00:00Z", 79.8).await;

    let automatic = priorities(&pool).await.unwrap()[0].sources.clone();
    assert_eq!(
        automatic[0], "manual",
        "la saisie la plus récente passe devant"
    );

    set_priority(&pool, "weight", &["withings", "manual"])
        .await
        .unwrap();
    let chosen = priorities(&pool).await.unwrap();
    assert_eq!(chosen[0].sources, vec!["withings", "manual"]);
    assert!(chosen[0].custom);

    set_priority(&pool, "weight", &[]).await.unwrap();
    let back = priorities(&pool).await.unwrap();
    assert_eq!(back[0].sources, automatic);
    assert!(!back[0].custom);
}

#[tokio::test]
async fn a_chosen_source_without_data_is_not_shown() {
    let pool = memory_pool().await;
    weigh(&pool, "withings", "2026-03-01T07:00:00Z", 80.0).await;
    weigh(&pool, "manual", "2026-03-02T09:00:00Z", 79.8).await;
    set_priority(&pool, "weight", &["oura", "withings", "manual"])
        .await
        .unwrap();

    let listed = priorities(&pool).await.unwrap();

    assert_eq!(listed[0].sources, vec!["withings", "manual"]);
}

#[test]
fn listing_the_priorities_never_loads_the_history_of_a_metric() {
    use crate::support::peak_alloc::peak_of;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let pool = runtime.block_on(async {
        let pool = memory_pool().await;
        sqlx::query(
            "WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < 99999) \
             INSERT INTO body_measures_legacy (measured_at, source, weight) \
             SELECT strftime('%Y-%m-%dT%H:%M:%SZ', '2020-01-01', '+' || i || ' minutes'), \
                    CASE WHEN i % 1000 = 0 THEN 'manual' ELSE 'withings' END, 70.0 + (i % 50) / 10.0 \
             FROM n",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    });

    let (listed, peak) = peak_of(|| runtime.block_on(priorities(&pool)).unwrap());

    let weight = listed.iter().find(|m| m.slug == "weight").expect("poids");
    assert_eq!(weight.sources.len(), 2);
    assert_eq!(weight.sources[0], "withings");
    assert!(peak < 256 * 1024, "pic de {peak} octets");
}
