use sqlx::SqlitePool;
use synk_lib::error::AppError;

async fn delete_audits(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = 'delete'")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn deleting_a_missing_medication_is_not_found_without_audit() {
    let pool = crate::common::memory_pool().await;

    let err = synk_lib::commands::medications::delete_medication_inner(&pool, 99)
        .await
        .unwrap_err();

    assert!(
        matches!(
            err,
            AppError::NotFound {
                entity: "medications",
                id: 99
            }
        ),
        "{err}"
    );
    assert_eq!(delete_audits(&pool).await, 0);
}

#[tokio::test]
async fn deleting_a_medication_removes_its_logs_and_is_audited() {
    let pool = crate::common::memory_pool().await;
    let id = sqlx::query("INSERT INTO medications (name) VALUES ('Paracetamol')")
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();
    sqlx::query(
        "INSERT INTO medication_logs (medication_id, taken_at, skipped) VALUES (?1, '2026-10-03T08:00:00Z', 0)",
    )
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();

    synk_lib::commands::medications::delete_medication_inner(&pool, id)
        .await
        .unwrap();

    let logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM medication_logs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(logs, 0);
    assert_eq!(delete_audits(&pool).await, 1);
}

#[tokio::test]
async fn a_builtin_measurement_type_cannot_be_deleted() {
    let pool = crate::common::memory_pool().await;
    let id = sqlx::query(
        "INSERT INTO measurement_types_legacy (slug, label, unit, category, sort_order, is_builtin) VALUES ('test_integre', 'Integre', 'cm', 'custom', 99, 1)",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_rowid();

    let err = synk_lib::commands::manual_measurements::delete_measurement_type_inner(&pool, id)
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Forbidden(_)), "{err}");
    assert_eq!(delete_audits(&pool).await, 0);
}
