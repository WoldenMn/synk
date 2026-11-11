use sqlx::SqlitePool;

use super::insert_insight;
use crate::error::AppError;
use crate::services::metrics::keep_preferred;
use crate::utils::dates::date_prefix;

pub const SQL: &str = "SELECT measured_at, source, systolic, diastolic FROM blood_pressure \
     WHERE measured_at >= date('now','-30 days') ORDER BY measured_at ASC";

pub async fn detect(pool: &SqlitePool) -> Result<i64, AppError> {
    let readings: Vec<(String, String, i64, i64)> = sqlx::query_as(SQL).fetch_all(pool).await?;
    let readings = keep_preferred(
        pool,
        "systolic",
        readings,
        |r| date_prefix(&r.0).to_string(),
        |r| r.1.clone(),
    )
    .await?;
    let Some(&(_, _, last_sys, last_dia)) = readings.last() else {
        return Ok(0);
    };
    let count = readings.len() as f64;
    let avg_sys = readings.iter().map(|r| r.2 as f64).sum::<f64>() / count;
    let avg_dia = readings.iter().map(|r| r.3 as f64).sum::<f64>() / count;
    if avg_sys <= 0.0 || (last_sys < 140 && last_dia < 90) {
        return Ok(0);
    }
    let inserted = insert_insight(
        pool,
        "anomaly",
        "danger",
        Some("blood_pressure"),
        "Hypertension détectée",
        &format!(
            "Dernière mesure: {}/{} mmHg (moyenne {:.0}/{:.0})",
            last_sys, last_dia, avg_sys, avg_dia
        ),
        Some(last_sys as f64),
        None,
    )
    .await?;
    Ok(i64::from(inserted))
}
