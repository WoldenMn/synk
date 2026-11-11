use sqlx::SqlitePool;

use super::insert_insight;
use crate::error::AppError;
use crate::services::metrics::keep_preferred;
use crate::utils::dates::date_prefix;

pub const SQL: &str = "SELECT timestamp, source, CAST(heart_rate AS REAL) FROM heart_records \
     WHERE heart_rate IS NOT NULL AND heart_rate > 30 \
     AND timestamp >= date('now','-30 days') \
     ORDER BY timestamp ASC";

pub async fn detect(pool: &SqlitePool) -> Result<i64, AppError> {
    let readings: Vec<(String, String, f64)> = sqlx::query_as(SQL).fetch_all(pool).await?;
    let readings = keep_preferred(
        pool,
        "heart_rate",
        readings,
        |r| date_prefix(&r.0).to_string(),
        |r| r.1.clone(),
    )
    .await?;
    let values: Vec<f64> = readings.into_iter().map(|r| r.2).collect();
    if values.len() < 10 {
        return Ok(0);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let std = var.sqrt();
    let Some(&last) = values.last() else {
        return Ok(0);
    };
    if std <= 0.0 || (last - mean) / std <= 2.0 {
        return Ok(0);
    }
    let inserted = insert_insight(
        pool,
        "anomaly",
        "warning",
        Some("heart_rate"),
        "FC anormalement élevée",
        &format!("Dernière mesure: {:.0} bpm (moyenne {:.0})", last, mean),
        Some(last),
        Some(last - mean),
    )
    .await?;
    Ok(i64::from(inserted))
}
