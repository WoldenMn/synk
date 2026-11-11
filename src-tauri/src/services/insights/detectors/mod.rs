pub mod activity_delta;
pub mod heart_rate_anomaly;
pub mod hypertension;
pub mod sleep_insufficient;
pub mod weight_trend;

use sqlx::SqlitePool;

// `too_many_arguments` accepte ici : les arguments sont logiquement independants.
// Une struct serait du sucre verbeux (YAGNI).
#[allow(clippy::too_many_arguments)]
pub async fn insert_insight(
    pool: &SqlitePool,
    kind: &str,
    severity: &str,
    metric: Option<&str>,
    title: &str,
    message: &str,
    value: Option<f64>,
    delta: Option<f64>,
) -> Result<bool, sqlx::Error> {
    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM insights \
         WHERE kind = ?1 AND metric IS ?2 AND date(generated_at) = date('now') LIMIT 1",
    )
    .bind(kind)
    .bind(metric)
    .fetch_optional(pool)
    .await?;
    if existing.is_some() {
        return Ok(false);
    }
    sqlx::query(
        "INSERT INTO insights (kind, severity, metric, title, message, value, delta) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(kind)
    .bind(severity)
    .bind(metric)
    .bind(title)
    .bind(message)
    .bind(value)
    .bind(delta)
    .execute(pool)
    .await?;
    Ok(true)
}
