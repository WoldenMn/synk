// B-5 : delete_alert atomique (alert_events supprimés avant health_alerts, dans une transaction)
use synk_lib::commands::alerts::delete_alert_inner;

#[tokio::test]
async fn delete_alert_removes_alert_and_its_events() {
    let pool = crate::common::memory_pool().await;

    // Inserer une alerte
    let alert_id: i64 = sqlx::query_scalar(
        "INSERT INTO health_alerts (metric_key, condition, threshold, label, enabled) \
         VALUES ('weight', 'above', 80.0, 'Test B5', 1) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Inserer 2 alert_events associes
    for val in [85.0_f64, 90.0] {
        sqlx::query(
            "INSERT INTO alert_events (alert_id, value, triggered_at) \
             VALUES (?1, ?2, '2026-01-01T10:00:00')",
        )
        .bind(alert_id)
        .bind(val)
        .execute(&pool)
        .await
        .unwrap();
    }

    // Precondition
    let before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alert_events WHERE alert_id = ?1")
        .bind(alert_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(before.0, 2, "2 events avant suppression");

    // Suppression atomique
    delete_alert_inner(&pool, alert_id).await.unwrap();

    // Postcondition : alerte et events a 0
    let alerts: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM health_alerts WHERE id = ?1")
        .bind(alert_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(alerts.0, 0, "alerte supprimee");

    let events: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alert_events WHERE alert_id = ?1")
        .bind(alert_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(events.0, 0, "events orphelins supprimes");
}
