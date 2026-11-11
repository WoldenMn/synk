use sqlx::{Pool, Sqlite};

use crate::error::AppError;
use crate::metric_repository::{
    legacy_query, source_of, unknown_metric, LegacyTable, MetricSource, PULSE_TWIN,
};

pub async fn read_metric_sources(
    pool: &Pool<Sqlite>,
    slug: &str,
) -> Result<Vec<(String, String)>, AppError> {
    let Some((def_id, source)) = source_of(pool, slug).await? else {
        return Err(unknown_metric(slug));
    };
    let mut found: Vec<(String, String)> = Vec::new();
    let mut merge = |rows: Vec<(String, String)>| {
        for (source, latest) in rows {
            match found.iter_mut().find(|(s, _)| *s == source) {
                Some(slot) if slot.1 < latest => slot.1 = latest,
                Some(_) => {}
                None => found.push((source, latest)),
            }
        }
    };
    if let MetricSource::Legacy(table) | MetricSource::Split(table) = source {
        merge(legacy_sources(pool, table, slug).await?);
    }
    if let MetricSource::Metrics | MetricSource::Split(_) = source {
        let split = matches!(source, MetricSource::Split(_));
        merge(metrics_sources(pool, def_id, split).await?);
    }
    Ok(found)
}

async fn legacy_sources(
    pool: &Pool<Sqlite>,
    table: LegacyTable,
    slug: &str,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    let Some(q) = legacy_query(table, slug) else {
        return Ok(Vec::new());
    };
    let sql = format!(
        "SELECT source, MAX(day) FROM (SELECT {}, {} AS day FROM {}) GROUP BY source",
        q.select, q.order_col, q.from_where
    );
    let mut query = sqlx::query_as::<_, (String, String)>(sqlx::AssertSqlSafe(sql));
    if q.binds_slug {
        query = query.bind(slug.to_string());
    }
    query.fetch_all(pool).await
}

async fn metrics_sources(
    pool: &Pool<Sqlite>,
    def_id: i64,
    without_twin: bool,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    let twin = if without_twin {
        format!(" AND {PULSE_TWIN}")
    } else {
        String::new()
    };
    let sql = format!(
        "SELECT source, MAX(measured_at) FROM metrics WHERE definition_id = ?{twin} GROUP BY source"
    );
    sqlx::query_as::<_, (String, String)>(sqlx::AssertSqlSafe(sql))
        .bind(def_id)
        .fetch_all(pool)
        .await
}
