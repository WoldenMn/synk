use sqlx::SqlitePool;

use super::insert_insight;
use crate::error::AppError;
use crate::services::metrics::keep_preferred;

pub async fn detect(pool: &SqlitePool) -> Result<i64, AppError> {
    let nights: Vec<(String, String, Option<i64>)> = sqlx::query_as(
        "SELECT date, source, duration_total FROM sleep_records \
         WHERE date >= date('now','-7 days') ORDER BY date DESC",
    )
    .fetch_all(pool)
    .await?;
    let nights = keep_preferred(
        pool,
        "sleep_score",
        nights,
        |n| n.0.clone(),
        |n| n.1.clone(),
    )
    .await?;
    let rows: Vec<Option<i64>> = nights.into_iter().take(3).map(|n| n.2).collect();
    let short = rows
        .iter()
        .all(|r| r.unwrap_or(0) > 0 && r.unwrap_or(0) < 21600);
    if rows.len() != 3 || !short {
        return Ok(0);
    }
    let avg = rows.iter().map(|r| r.unwrap_or(0)).sum::<i64>() / 3;
    let avg_h = avg as f64 / 3600.0;
    let inserted = insert_insight(
        pool,
        "anomaly",
        "warning",
        Some("sleep"),
        "Sommeil insuffisant",
        &format!("Moyenne de {:.1}h sur les 3 dernières nuits", avg_h),
        Some(avg_h),
        None,
    )
    .await?;
    Ok(i64::from(inserted))
}
