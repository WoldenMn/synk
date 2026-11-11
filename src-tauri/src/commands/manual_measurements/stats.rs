use crate::error::AppError;
use crate::utils::dates::day_range;
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::{BTreeMap, HashMap};
use tauri::State;

#[derive(Debug, Serialize, PartialEq)]
pub struct MeasurementStats {
    pub latest_value: Option<f64>,
    pub latest_date: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub avg_value: Option<f64>,
    pub count: i64,
    pub prev_value: Option<f64>,
    pub percent_change: Option<f64>,
}

pub fn summarize(points: &[(String, f64)], prev: Option<f64>) -> MeasurementStats {
    let count = points.len() as i64;
    let min_value = points.iter().map(|p| p.1).reduce(f64::min);
    let max_value = points.iter().map(|p| p.1).reduce(f64::max);
    let avg_value = (count > 0).then(|| points.iter().map(|p| p.1).sum::<f64>() / count as f64);
    let latest = points.last();
    let percent_change = match (latest, prev) {
        (Some(l), Some(p)) if p != 0.0 => Some(((l.1 - p) / p) * 100.0),
        _ => None,
    };
    MeasurementStats {
        latest_value: latest.map(|l| l.1),
        latest_date: latest.map(|l| l.0.clone()),
        min_value,
        max_value,
        avg_value,
        count,
        prev_value: prev,
        percent_change,
    }
}

pub async fn get_measurement_stats_inner(
    pool: &SqlitePool,
    type_id: i64,
    from: &str,
    to: &str,
) -> Result<MeasurementStats, AppError> {
    let range = day_range(from, to)?;
    let (from_d, to_d) = (range.from, range.before);
    let points: Vec<(String, f64)> = sqlx::query_as(
        "SELECT measured_at, value FROM manual_measurements \
         WHERE measurement_type_id = ?1 AND measured_at >= ?2 AND measured_at < ?3 \
         ORDER BY measured_at ASC",
    )
    .bind(type_id)
    .bind(&from_d)
    .bind(&to_d)
    .fetch_all(pool)
    .await?;
    let prev: Option<f64> = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT value FROM manual_measurements \
         WHERE measurement_type_id = ?1 AND measured_at < ?2 \
         ORDER BY measured_at DESC LIMIT 1",
    )
    .bind(type_id)
    .bind(&from_d)
    .fetch_optional(pool)
    .await?
    .flatten();
    Ok(summarize(&points, prev))
}

#[tauri::command]
pub async fn get_measurement_stats(
    pool: State<'_, SqlitePool>,
    type_id: i64,
    from: String,
    to: String,
) -> Result<MeasurementStats, AppError> {
    get_measurement_stats_inner(pool.inner(), type_id, &from, &to).await
}

#[derive(Debug, Serialize)]
pub struct MeasurementOverview {
    pub type_id: i64,
    pub stats: MeasurementStats,
    pub values: Vec<f64>,
}

pub async fn get_measurement_overview_inner(
    pool: &SqlitePool,
    from: &str,
    to: &str,
) -> Result<Vec<MeasurementOverview>, AppError> {
    let range = day_range(from, to)?;
    let (from_d, to_d) = (range.from, range.before);
    let rows: Vec<(i64, String, f64)> = sqlx::query_as(
        "SELECT measurement_type_id, measured_at, value FROM manual_measurements \
         WHERE measured_at >= ?1 AND measured_at < ?2 \
         ORDER BY measurement_type_id, measured_at ASC",
    )
    .bind(&from_d)
    .bind(&to_d)
    .fetch_all(pool)
    .await?;
    let prevs: Vec<(i64, Option<f64>)> = sqlx::query_as(
        "SELECT measurement_type_id, value FROM ( \
           SELECT measurement_type_id, value, \
                  ROW_NUMBER() OVER (PARTITION BY measurement_type_id ORDER BY measured_at DESC) AS rang \
           FROM manual_measurements WHERE measured_at < ?1) \
         WHERE rang = 1",
    )
    .bind(&from_d)
    .fetch_all(pool)
    .await?;
    let prev_by_type: HashMap<i64, Option<f64>> = prevs.into_iter().collect();
    let mut by_type: BTreeMap<i64, Vec<(String, f64)>> = BTreeMap::new();
    for (type_id, at, value) in rows {
        by_type.entry(type_id).or_default().push((at, value));
    }
    for type_id in prev_by_type.keys() {
        by_type.entry(*type_id).or_default();
    }
    Ok(by_type
        .into_iter()
        .map(|(type_id, points)| MeasurementOverview {
            type_id,
            values: points.iter().map(|p| p.1).collect(),
            stats: summarize(&points, prev_by_type.get(&type_id).copied().flatten()),
        })
        .collect())
}

#[tauri::command]
pub async fn get_measurement_overview(
    pool: State<'_, SqlitePool>,
    from: String,
    to: String,
) -> Result<Vec<MeasurementOverview>, AppError> {
    get_measurement_overview_inner(pool.inner(), &from, &to).await
}
