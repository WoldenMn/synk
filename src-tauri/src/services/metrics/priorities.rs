use std::collections::HashMap;

use serde::Serialize;
use sqlx::SqlitePool;

use super::present::read_metric_sources;
use super::priority::priority_of;
use super::select::{load_tiebreak, ranking};
use crate::error::AppError;
use crate::utils::dates::date_prefix;

#[derive(Debug, Serialize)]
pub struct MetricPriority {
    pub slug: String,
    pub label: String,
    pub sources: Vec<String>,
    pub custom: bool,
}

pub async fn priorities(pool: &SqlitePool) -> Result<Vec<MetricPriority>, AppError> {
    let definitions: Vec<(i64, String, String)> =
        sqlx::query_as("SELECT id, slug, label FROM metric_definitions ORDER BY sort_order")
            .fetch_all(pool)
            .await?;
    let sort = load_tiebreak(pool).await?;
    let mut listed = Vec::new();
    for (id, slug, label) in definitions {
        let found = read_metric_sources(pool, &slug).await?;
        if found.len() < 2 {
            continue;
        }
        let latest: HashMap<&str, &str> = found
            .iter()
            .map(|(source, at)| (source.as_str(), date_prefix(at)))
            .collect();
        let chosen = priority_of(pool, id).await?;
        let sources = ranking(&chosen, &latest, &sort)
            .into_iter()
            .filter(|s| latest.contains_key(s.as_str()))
            .collect();
        listed.push(MetricPriority {
            slug,
            label,
            sources,
            custom: !chosen.is_empty(),
        });
    }
    Ok(listed)
}
