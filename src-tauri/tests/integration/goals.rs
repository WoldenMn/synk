use synk_lib::commands::goals::{upsert_goal_inner, UpsertGoalInput};

fn input(direction: &str, target: f64) -> UpsertGoalInput {
    UpsertGoalInput {
        metric_key: "weight".into(),
        target_value: target,
        direction: direction.into(),
    }
}

#[tokio::test]
async fn refuses_an_unknown_direction() {
    let pool = crate::common::memory_pool().await;
    let err = upsert_goal_inner(&pool, &input("below", 70.0))
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("sens d'objectif inconnu : below"),
        "obtenu : {err}"
    );
}

#[tokio::test]
async fn refuses_a_non_positive_target() {
    let pool = crate::common::memory_pool().await;
    let err = upsert_goal_inner(&pool, &input("decrease", 0.0))
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("valeur cible invalide"),
        "obtenu : {err}"
    );
}

#[tokio::test]
async fn stores_a_valid_goal() {
    let pool = crate::common::memory_pool().await;
    upsert_goal_inner(&pool, &input("decrease", 70.0))
        .await
        .unwrap();
    let row: (String, f64) =
        sqlx::query_as("SELECT direction, target_value FROM goals WHERE metric_key = 'weight'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row, ("decrease".to_string(), 70.0));
}
