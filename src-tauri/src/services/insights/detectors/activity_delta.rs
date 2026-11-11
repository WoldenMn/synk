use sqlx::SqlitePool;

use super::insert_insight;
use crate::error::AppError;
use crate::metric_repository::MetricPoint;
use crate::services::metrics::select_series;

fn average(points: &[MetricPoint]) -> f64 {
    if points.is_empty() {
        return 0.0;
    }
    points.iter().map(|p| p.value).sum::<f64>() / points.len() as f64
}

pub async fn detect(pool: &SqlitePool) -> Result<i64, AppError> {
    let today = chrono::Utc::now().date_naive();
    let day = |back: i64| (today - chrono::Duration::days(back)).to_string();
    let curr = average(&select_series(pool, "steps", &day(7), &today.to_string()).await?);
    let prev = average(&select_series(pool, "steps", &day(14), &day(8)).await?);
    if prev <= 100.0 || curr <= 100.0 {
        return Ok(0);
    }
    let pct = ((curr - prev) / prev) * 100.0;
    if pct.abs() < 15.0 {
        return Ok(0);
    }
    let (sev, title) = if pct > 0.0 {
        ("info", "Activité en hausse")
    } else {
        ("warning", "Activité en baisse")
    };
    let inserted = insert_insight(
        pool,
        "trend",
        sev,
        Some("steps"),
        title,
        &format!(
            "{:+.0}% de pas vs semaine dernière ({:.0} → {:.0})",
            pct, prev, curr
        ),
        Some(curr),
        Some(curr - prev),
    )
    .await?;
    Ok(i64::from(inserted))
}
