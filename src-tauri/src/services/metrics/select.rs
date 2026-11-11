use std::collections::HashMap;

use sqlx::SqlitePool;

use super::priority::priority_of;
use crate::error::AppError;
use crate::metric_repository::{
    read_latest_resolved, read_metric_series, read_series_resolved, source_of, unknown_metric,
    MetricPoint, MetricSource,
};
use crate::utils::dates::{date_prefix, day_range};

pub struct Tiebreak {
    sort: HashMap<String, i64>,
    connected_at: HashMap<String, String>,
}

pub async fn load_tiebreak(pool: &SqlitePool) -> Result<Tiebreak, sqlx::Error> {
    let sort: Vec<(String, i64)> = sqlx::query_as("SELECT id, sort_order FROM sources")
        .fetch_all(pool)
        .await?;
    let connected: Vec<(String, String)> =
        sqlx::query_as("SELECT source_id, created_at FROM source_auth")
            .fetch_all(pool)
            .await?;
    Ok(Tiebreak {
        sort: sort.into_iter().collect(),
        connected_at: connected.into_iter().collect(),
    })
}

fn connection_order(tie: &Tiebreak, a: &str, b: &str) -> std::cmp::Ordering {
    match (tie.connected_at.get(a), tie.connected_at.get(b)) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

pub(super) fn latest_days(points: &[MetricPoint]) -> HashMap<&str, &str> {
    let mut latest_day: HashMap<&str, &str> = HashMap::new();
    for p in points {
        let day = date_prefix(&p.measured_at);
        let slot = latest_day.entry(p.source.as_str()).or_insert(day);
        if day > *slot {
            *slot = day;
        }
    }
    latest_day
}

pub(super) fn ranking(
    priority: &[String],
    latest_day: &HashMap<&str, &str>,
    tie: &Tiebreak,
) -> Vec<String> {
    let mut others: Vec<&str> = latest_day
        .keys()
        .copied()
        .filter(|s| !priority.iter().any(|p| p == s))
        .collect();
    others.sort_by(|a, b| {
        latest_day[b]
            .cmp(latest_day[a])
            .then_with(|| connection_order(tie, a, b))
            .then_with(|| {
                tie.sort
                    .get(*a)
                    .unwrap_or(&i64::MAX)
                    .cmp(tie.sort.get(*b).unwrap_or(&i64::MAX))
            })
            .then_with(|| a.cmp(b))
    });
    priority
        .iter()
        .cloned()
        .chain(others.into_iter().map(str::to_string))
        .collect()
}

pub(super) fn one_source_per_day(points: Vec<MetricPoint>, order: &[String]) -> Vec<MetricPoint> {
    let position = |source: &str| order.iter().position(|s| s == source).unwrap_or(usize::MAX);
    let mut best: HashMap<String, usize> = HashMap::new();
    for p in &points {
        let rank = position(&p.source);
        let slot = best
            .entry(date_prefix(&p.measured_at).to_string())
            .or_insert(rank);
        if rank < *slot {
            *slot = rank;
        }
    }
    let mut chosen: Vec<MetricPoint> = points
        .into_iter()
        .filter(|p| best.get(date_prefix(&p.measured_at)) == Some(&position(&p.source)))
        .collect();
    chosen.dedup_by(|later, first| later.measured_at == first.measured_at);
    chosen
}

pub struct Resolved {
    slug: String,
    definition_id: i64,
    source: MetricSource,
    priority: Vec<String>,
}

pub async fn resolve(pool: &SqlitePool, slug: &str) -> Result<Option<Resolved>, AppError> {
    let Some((definition_id, source)) = source_of(pool, slug).await? else {
        return Ok(None);
    };
    let priority = priority_of(pool, definition_id).await?;
    Ok(Some(Resolved {
        slug: slug.to_string(),
        definition_id,
        source,
        priority,
    }))
}

impl Resolved {
    pub async fn series(
        &self,
        pool: &SqlitePool,
        tie: &Tiebreak,
        from: &str,
        to: &str,
    ) -> Result<Vec<MetricPoint>, AppError> {
        let range = day_range(from, to)?;
        let points = read_series_resolved(
            pool,
            self.definition_id,
            self.source,
            &self.slug,
            &range.from,
            &range.before,
        )
        .await?;
        if points.is_empty() {
            return Ok(points);
        }
        let order = ranking(&self.priority, &latest_days(&points), tie);
        Ok(one_source_per_day(points, &order))
    }

    pub async fn latest_before(
        &self,
        pool: &SqlitePool,
        tie: &Tiebreak,
        before_day: &str,
    ) -> Result<Option<MetricPoint>, AppError> {
        let Some(last) = read_latest_resolved(
            pool,
            self.definition_id,
            self.source,
            &self.slug,
            Some(before_day),
        )
        .await?
        else {
            return Ok(None);
        };
        let day = date_prefix(&last.measured_at).to_string();
        Ok(self.series(pool, tie, &day, &day).await?.pop())
    }

    pub async fn latest(
        &self,
        pool: &SqlitePool,
        tie: &Tiebreak,
    ) -> Result<Option<MetricPoint>, AppError> {
        self.latest_before(pool, tie, "9999-12-31").await
    }
}

pub async fn select_series(
    pool: &SqlitePool,
    slug: &str,
    from: &str,
    to: &str,
) -> Result<Vec<MetricPoint>, AppError> {
    let Some(metric) = resolve(pool, slug).await? else {
        return Err(unknown_metric(slug));
    };
    metric
        .series(pool, &load_tiebreak(pool).await?, from, to)
        .await
}

pub async fn select_latest(pool: &SqlitePool, slug: &str) -> Result<Option<MetricPoint>, AppError> {
    select_latest_before(pool, slug, "9999-12-31").await
}

pub async fn select_latest_before(
    pool: &SqlitePool,
    slug: &str,
    before_day: &str,
) -> Result<Option<MetricPoint>, AppError> {
    let Some(metric) = resolve(pool, slug).await? else {
        return Ok(None);
    };
    metric
        .latest_before(pool, &load_tiebreak(pool).await?, before_day)
        .await
}

pub async fn select_secondary(
    pool: &SqlitePool,
    slug: &str,
    from: &str,
    to: &str,
) -> Result<Vec<MetricPoint>, AppError> {
    let Some((definition_id, _)) = source_of(pool, slug).await? else {
        return Err(unknown_metric(slug));
    };
    let points = read_metric_series(pool, slug, from, to).await?;
    if points.is_empty() {
        return Ok(points);
    }
    let priority = priority_of(pool, definition_id).await?;
    let order = ranking(
        &priority,
        &latest_days(&points),
        &load_tiebreak(pool).await?,
    );
    let position = |source: &str| order.iter().position(|s| s == source).unwrap_or(usize::MAX);
    let mut best: HashMap<String, usize> = HashMap::new();
    for p in &points {
        let slot = best
            .entry(date_prefix(&p.measured_at).to_string())
            .or_insert(usize::MAX);
        *slot = (*slot).min(position(&p.source));
    }
    Ok(points
        .into_iter()
        .filter(|p| best.get(date_prefix(&p.measured_at)) != Some(&position(&p.source)))
        .collect())
}

pub async fn keep_preferred<T>(
    pool: &SqlitePool,
    slug: &str,
    rows: Vec<T>,
    day_of: impl Fn(&T) -> String,
    source_of_row: impl Fn(&T) -> String,
) -> Result<Vec<T>, AppError> {
    let Some((definition_id, _)) = source_of(pool, slug).await? else {
        return Ok(rows);
    };
    let points: Vec<MetricPoint> = rows
        .iter()
        .map(|r| MetricPoint {
            measured_at: day_of(r),
            value: 0.0,
            source: source_of_row(r),
        })
        .collect();
    let priority = priority_of(pool, definition_id).await?;
    let order = ranking(
        &priority,
        &latest_days(&points),
        &load_tiebreak(pool).await?,
    );
    let position = |source: &str| order.iter().position(|s| s == source).unwrap_or(usize::MAX);
    let mut best: HashMap<String, usize> = HashMap::new();
    for p in &points {
        let slot = best.entry(p.measured_at.clone()).or_insert(usize::MAX);
        *slot = (*slot).min(position(&p.source));
    }
    Ok(rows
        .into_iter()
        .filter(|r| best.get(&day_of(r)) == Some(&position(&source_of_row(r))))
        .collect())
}
