use crate::error::AppError;
pub(crate) use crate::metric_legacy::legacy_query;
pub use crate::metric_legacy::series_sql;
use crate::metric_legacy::{body_composition_column, legacy_latest, legacy_series};
use crate::utils::dates::day_range;
use serde::Serialize;
use sqlx::{Pool, Sqlite};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MetricPoint {
    pub measured_at: String,
    pub value: f64,
    pub source: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegacyTable {
    BodyMeasures,
    ManualMeasurements,
    Vitals,
    BloodPressure,
    SleepRecords,
    Activity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricSource {
    Legacy(LegacyTable),
    Metrics,
    Split(LegacyTable),
}

pub fn classify(category: &str, slug: &str) -> Option<MetricSource> {
    use LegacyTable::*;
    match category {
        "body_composition" => {
            body_composition_column(slug).map(|_| MetricSource::Legacy(BodyMeasures))
        }
        "circumference" | "custom" => Some(MetricSource::Legacy(ManualMeasurements)),
        "body_scan" => Some(MetricSource::Legacy(Vitals)),
        "vital_bp" => match slug {
            "systolic" | "diastolic" => Some(MetricSource::Legacy(BloodPressure)),
            "heart_rate" => Some(MetricSource::Split(BloodPressure)),
            _ => None,
        },
        "vital_sleep" => Some(MetricSource::Legacy(SleepRecords)),
        "vital_activity" => Some(MetricSource::Legacy(Activity)),
        _ => Some(MetricSource::Metrics),
    }
}

pub fn unknown_metric(slug: &str) -> AppError {
    AppError::NotFoundByKey {
        entity: "metrique",
        key: slug.to_string(),
    }
}

pub async fn source_of(
    pool: &Pool<Sqlite>,
    slug: &str,
) -> Result<Option<(i64, MetricSource)>, sqlx::Error> {
    let row: Option<(i64, String)> =
        sqlx::query_as("SELECT id, category FROM metric_definitions WHERE slug = ?")
            .bind(slug)
            .fetch_optional(pool)
            .await?;
    Ok(row.and_then(|(id, category)| classify(&category, slug).map(|s| (id, s))))
}

pub(crate) const PULSE_TWIN: &str = "NOT EXISTS (SELECT 1 FROM blood_pressure bp WHERE bp.measured_at = metrics.measured_at AND bp.heart_rate IS NOT NULL)";

async fn metrics_series(
    pool: &Pool<Sqlite>,
    def_id: i64,
    from: &str,
    to: &str,
    without_twin: bool,
) -> Result<Vec<MetricPoint>, sqlx::Error> {
    let sql = if without_twin {
        format!("SELECT measured_at, value, source FROM metrics WHERE definition_id = ? AND measured_at >= ? AND measured_at < ? AND {PULSE_TWIN} ORDER BY measured_at ASC")
    } else {
        "SELECT measured_at, value, source FROM metrics WHERE definition_id = ? AND measured_at >= ? AND measured_at < ? ORDER BY measured_at ASC".to_string()
    };
    sqlx::query_as::<_, MetricPoint>(sqlx::AssertSqlSafe(sql))
        .bind(def_id)
        .bind(from.to_string())
        .bind(to.to_string())
        .fetch_all(pool)
        .await
}

pub async fn read_metric_series(
    pool: &Pool<Sqlite>,
    slug: &str,
    from: &str,
    to: &str,
) -> Result<Vec<MetricPoint>, AppError> {
    let range = day_range(from, to)?;
    let Some((def_id, source)) = source_of(pool, slug).await? else {
        return Err(unknown_metric(slug));
    };
    read_series_resolved(pool, def_id, source, slug, &range.from, &range.before).await
}

pub(crate) async fn read_series_resolved(
    pool: &Pool<Sqlite>,
    def_id: i64,
    source: MetricSource,
    slug: &str,
    from_d: &str,
    to_d: &str,
) -> Result<Vec<MetricPoint>, AppError> {
    let points = match source {
        MetricSource::Legacy(table) => legacy_series(pool, table, slug, from_d, to_d).await?,
        MetricSource::Metrics => metrics_series(pool, def_id, from_d, to_d, false).await?,
        MetricSource::Split(table) => {
            let mut points = legacy_series(pool, table, slug, from_d, to_d).await?;
            points.extend(metrics_series(pool, def_id, from_d, to_d, true).await?);
            points.sort_by(|a, b| a.measured_at.cmp(&b.measured_at));
            points
        }
    };
    Ok(points)
}

pub async fn read_metric_latest(
    pool: &Pool<Sqlite>,
    slug: &str,
) -> Result<Option<MetricPoint>, sqlx::Error> {
    read_metric_latest_before(pool, slug, None).await
}

pub async fn read_metric_latest_before(
    pool: &Pool<Sqlite>,
    slug: &str,
    before: Option<&str>,
) -> Result<Option<MetricPoint>, sqlx::Error> {
    let Some((def_id, source)) = source_of(pool, slug).await? else {
        return Ok(None);
    };
    read_latest_resolved(pool, def_id, source, slug, before).await
}

pub(crate) async fn read_latest_resolved(
    pool: &Pool<Sqlite>,
    def_id: i64,
    source: MetricSource,
    slug: &str,
    before: Option<&str>,
) -> Result<Option<MetricPoint>, sqlx::Error> {
    let from_metrics = || {
        let bound = before.map_or("", |_| " AND measured_at < ?");
        let sql = format!(
            "SELECT measured_at, value, source FROM metrics WHERE definition_id = ?{bound} ORDER BY measured_at DESC LIMIT 1"
        );
        let mut query = sqlx::query_as::<_, MetricPoint>(sqlx::AssertSqlSafe(sql)).bind(def_id);
        if let Some(day) = before {
            query = query.bind(day.to_string());
        }
        query.fetch_optional(pool)
    };
    match source {
        MetricSource::Legacy(table) => legacy_latest(pool, table, slug, before).await,
        MetricSource::Metrics => from_metrics().await,
        MetricSource::Split(table) => {
            let legacy = legacy_latest(pool, table, slug, before).await?;
            let own = from_metrics().await?;
            Ok(match (legacy, own) {
                (Some(l), Some(m)) if m.measured_at > l.measured_at => Some(m),
                (Some(l), _) => Some(l),
                (None, m) => m,
            })
        }
    }
}
