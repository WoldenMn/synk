use sqlx::SqlitePool;
use synk_lib::commands::clothing_sizes::{
    delete_clothing_size_inner, get_clothing_inputs_inner, get_clothing_sizes_inner,
    save_estimated_sizes_inner, set_clothing_size_inner, ClothingInputs, ClothingSize,
    EstimatedSize,
};
use synk_lib::db::bootstrap_schema;

#[tokio::test]
async fn test_clothing_sizes_table_exists() {
    let pool = crate::common::memory_pool().await;
    let name: Option<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='clothing_sizes'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert_eq!(name.as_deref(), Some("clothing_sizes"));
}

#[tokio::test]
async fn test_inseam_seeded_once_and_idempotent() {
    let pool = crate::common::memory_pool().await;
    // Re-bootstrap : doit rester idempotent (pas de doublon).
    bootstrap_schema(&pool).await.unwrap();

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM measurement_types_legacy WHERE slug = 'inseam'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1, "inseam doit exister exactement une fois");

    let is_builtin: i64 =
        sqlx::query_scalar("SELECT is_builtin FROM measurement_types_legacy WHERE slug = 'inseam'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(is_builtin, 1, "inseam est un type built-in");
}

async fn insert_measure(pool: &SqlitePool, slug: &str, value: f64) {
    sqlx::query(
        "INSERT INTO manual_measurements_legacy (measurement_type_id, measured_at, value) \
         SELECT id, '2026-06-01T08:00:00Z', ?2 FROM measurement_types_legacy WHERE slug = ?1",
    )
    .bind(slug)
    .bind(value)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn test_clothing_inputs_empty_then_latest() {
    let pool = crate::common::memory_pool().await;

    let empty: ClothingInputs = get_clothing_inputs_inner(&pool).await.unwrap();
    assert_eq!(empty.chest, None);
    assert_eq!(empty.inseam, None);

    insert_measure(&pool, "chest", 90.0).await;
    insert_measure(&pool, "chest", 98.0).await;
    insert_measure(&pool, "neck", 40.0).await;
    insert_measure(&pool, "waist", 84.0).await;
    insert_measure(&pool, "inseam", 84.0).await;

    let got: ClothingInputs = get_clothing_inputs_inner(&pool).await.unwrap();
    assert_eq!(got.chest, Some(98.0));
    assert_eq!(got.neck, Some(40.0));
    assert_eq!(got.waist, Some(84.0));
    assert_eq!(got.inseam, Some(84.0));
}

#[tokio::test]
async fn test_clothing_sizes_crud() {
    let pool = crate::common::memory_pool().await;

    assert!(get_clothing_sizes_inner(&pool).await.unwrap().is_empty());

    set_clothing_size_inner(&pool, "jacket", "52", "override")
        .await
        .unwrap();
    set_clothing_size_inner(&pool, "shoes", "43", "manual")
        .await
        .unwrap();
    let rows: Vec<ClothingSize> = get_clothing_sizes_inner(&pool).await.unwrap();
    assert_eq!(rows.len(), 2);

    set_clothing_size_inner(&pool, "jacket", "50", "override")
        .await
        .unwrap();
    let rows = get_clothing_sizes_inner(&pool).await.unwrap();
    assert_eq!(rows.len(), 2, "upsert ne cree pas de doublon");
    let jacket = rows.iter().find(|r| r.category == "jacket").unwrap();
    assert_eq!(jacket.value, "50");

    assert!(set_clothing_size_inner(&pool, "jacket", "50", "bogus")
        .await
        .is_err());

    delete_clothing_size_inner(&pool, "jacket").await.unwrap();
    let rows = get_clothing_sizes_inner(&pool).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].category, "shoes");
}

#[tokio::test]
async fn saves_estimated_sizes() {
    let pool = crate::common::memory_pool().await;
    save_estimated_sizes_inner(
        &pool,
        vec![
            EstimatedSize {
                category: "top".into(),
                value: "L".into(),
            },
            EstimatedSize {
                category: "pants".into(),
                value: "48".into(),
            },
        ],
    )
    .await
    .unwrap();

    let rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT category, value, source FROM clothing_sizes ORDER BY category")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|(_, _, source)| source == "estimated"));
}

#[tokio::test]
async fn estimated_never_overwrites_a_manual_size() {
    let pool = crate::common::memory_pool().await;
    set_clothing_size_inner(&pool, "top", "XL", "manual")
        .await
        .unwrap();

    save_estimated_sizes_inner(
        &pool,
        vec![EstimatedSize {
            category: "top".into(),
            value: "L".into(),
        }],
    )
    .await
    .unwrap();

    let (value, source): (String, String) =
        sqlx::query_as("SELECT value, source FROM clothing_sizes WHERE category = 'top'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(value, "XL", "la correction manuelle doit survivre");
    assert_eq!(source, "manual");
}

#[tokio::test]
async fn estimated_updates_a_previous_estimate() {
    let pool = crate::common::memory_pool().await;
    save_estimated_sizes_inner(
        &pool,
        vec![EstimatedSize {
            category: "top".into(),
            value: "L".into(),
        }],
    )
    .await
    .unwrap();
    save_estimated_sizes_inner(
        &pool,
        vec![EstimatedSize {
            category: "top".into(),
            value: "M".into(),
        }],
    )
    .await
    .unwrap();

    let value: String =
        sqlx::query_scalar("SELECT value FROM clothing_sizes WHERE category = 'top'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(value, "M");
}
