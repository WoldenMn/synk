use sqlx::SqlitePool;
use synk_lib::commands::manual_measurements::{
    get_measurement_overview_inner, get_measurement_stats_inner,
};

async fn setup() -> (SqlitePool, i64, i64) {
    let pool = crate::common::memory_pool().await;
    let mut ids = Vec::new();
    for slug in ["test_bras", "test_mollet"] {
        let id = sqlx::query(
            "INSERT INTO measurement_types_legacy (slug, label, unit, category, sort_order, is_builtin) \
             VALUES (?1, ?1, 'cm', 'custom', 900, 0)",
        )
        .bind(slug)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();
        ids.push(id);
    }
    let rows = [
        (ids[0], "2026-08-20T08:00:00Z", 80.0),
        (ids[0], "2026-09-02T08:00:00Z", 81.0),
        (ids[0], "2026-09-10T08:00:00Z", 79.5),
        (ids[0], "2026-09-20T08:00:00Z", 82.0),
        (ids[1], "2026-09-05T08:00:00Z", 37.0),
        (ids[1], "2026-09-25T08:00:00Z", 36.5),
    ];
    for (type_id, at, value) in rows {
        sqlx::query(
            "INSERT INTO manual_measurements_legacy (measurement_type_id, measured_at, value) VALUES (?1, ?2, ?3)",
        )
        .bind(type_id)
        .bind(at)
        .bind(value)
        .execute(&pool)
        .await
        .unwrap();
    }
    (pool, ids[0], ids[1])
}

#[tokio::test]
async fn overview_gives_the_same_figures_as_the_detail_in_one_read() {
    let (pool, arm, calf) = setup().await;
    let (from, to) = ("2026-09-01", "2026-09-30");

    let overview = get_measurement_overview_inner(&pool, from, to)
        .await
        .unwrap();

    for type_id in [arm, calf] {
        let entry = overview
            .iter()
            .find(|o| o.type_id == type_id)
            .expect("type present");
        let detail = get_measurement_stats_inner(&pool, type_id, from, to)
            .await
            .unwrap();
        assert_eq!(entry.stats, detail, "type {type_id}");
    }
    let arm_entry = overview.iter().find(|o| o.type_id == arm).unwrap();
    assert_eq!(arm_entry.values, vec![81.0, 79.5, 82.0]);
    assert_eq!(arm_entry.stats.prev_value, Some(80.0));
    assert_eq!(arm_entry.stats.count, 3);
}
