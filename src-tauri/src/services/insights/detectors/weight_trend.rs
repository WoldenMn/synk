use sqlx::SqlitePool;

use super::insert_insight;
use crate::error::AppError;
use crate::services::metrics::select_series;

pub async fn detect(pool: &SqlitePool) -> Result<i64, AppError> {
    let today = chrono::Utc::now().date_naive();
    let from = (today - chrono::Duration::days(30)).to_string();
    let weights = select_series(pool, "weight", &from, &today.to_string()).await?;
    if weights.len() < 7 {
        return Ok(0);
    }
    let (Some(first_row), Some(last_row)) = (weights.first(), weights.last()) else {
        return Ok(0);
    };
    let delta = last_row.value - first_row.value;
    if delta.abs() <= 0.5 {
        return Ok(0);
    }
    let (severity, title) = if delta < 0.0 {
        ("info", "Poids en baisse")
    } else {
        ("warning", "Poids en hausse")
    };
    let inserted = insert_insight(
        pool,
        "trend",
        severity,
        Some("weight"),
        title,
        &format!("Variation de {:+.1} kg sur 30 jours", delta),
        Some(last_row.value),
        Some(delta),
    )
    .await?;
    Ok(i64::from(inserted))
}
