use crate::error::AppError;
use crate::metric_repository::unknown_metric;
use crate::services::metrics::{load_tiebreak, resolve, Resolved, Tiebreak};
use crate::utils::dates::day_range;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct TrendData {
    pub current_value: Option<f64>,
    pub previous_value: Option<f64>,
    pub percent_change: Option<f64>,
    pub direction: String,
}

pub async fn run_metric_trend(
    pool: &SqlitePool,
    metric: &str,
    from: &str,
    to: &str,
) -> Result<TrendData, AppError> {
    day_range(from, to)?;
    check_trend_metric(metric)?;
    let Some(resolved) = resolve(pool, metric).await? else {
        return Err(unknown_metric(metric));
    };
    metric_trend(pool, &resolved, &load_tiebreak(pool).await?, from, to).await
}

pub fn check_trend_metric(metric: &str) -> Result<(), AppError> {
    if matches!(metric, "weight" | "systolic" | "sleep_score" | "steps") {
        return Ok(());
    }
    Err(AppError::Validation(format!(
        "Tendance non disponible pour {metric}"
    )))
}

pub async fn metric_trend(
    pool: &SqlitePool,
    metric: &Resolved,
    tie: &Tiebreak,
    from: &str,
    to: &str,
) -> Result<TrendData, AppError> {
    let range = day_range(from, to)?;
    let (from_d, to_d) = (range.from.as_str(), range.before);

    let current = metric
        .series(pool, tie, from_d, &to_d)
        .await?
        .last()
        .map(|p| p.value);
    let previous = metric
        .latest_before(pool, tie, from_d)
        .await?
        .map(|p| p.value);

    let (percent_change, direction) = match (current, previous) {
        (Some(c), Some(p)) if p != 0.0 => {
            let pct = ((c - p) / p) * 100.0;
            let dir = if pct > 0.0 {
                "up"
            } else if pct < 0.0 {
                "down"
            } else {
                "stable"
            };
            (Some(pct), dir.to_string())
        }
        _ => (None, "stable".to_string()),
    };

    Ok(TrendData {
        current_value: current,
        previous_value: previous,
        percent_change,
        direction,
    })
}
