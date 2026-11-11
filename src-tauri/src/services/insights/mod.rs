pub mod detectors;

use crate::error::AppError;
use crate::validation::check_limit;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Insight {
    pub id: i64,
    pub kind: String,
    pub severity: String,
    pub metric: Option<String>,
    pub title: String,
    pub message: String,
    pub value: Option<f64>,
    pub delta: Option<f64>,
    pub generated_at: String,
    pub seen: i64,
}

pub async fn recent(pool: &SqlitePool, limit: Option<i64>) -> Result<Vec<Insight>, AppError> {
    let lim = check_limit(limit, 50)?;
    let rows = sqlx::query_as::<_, Insight>(
        "SELECT id, kind, severity, metric, title, message, value, delta, generated_at, seen FROM insights ORDER BY generated_at DESC LIMIT ?1",
    )
    .bind(lim)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn mark_seen(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::query("UPDATE insights SET seen = 1 WHERE seen = 0")
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, Default, Serialize)]
pub struct InsightReport {
    pub generated: i64,
    pub failed: Vec<String>,
}

pub async fn run_generation(pool: &SqlitePool) -> InsightReport {
    if let Err(e) =
        sqlx::query("DELETE FROM insights WHERE generated_at < datetime('now', '-30 days')")
            .execute(pool)
            .await
    {
        log::warn!("insights purge: {e}");
    }

    let mut report = InsightReport::default();
    tally(
        &mut report,
        "poids",
        detectors::weight_trend::detect(pool).await,
    );
    tally(
        &mut report,
        "sommeil",
        detectors::sleep_insufficient::detect(pool).await,
    );
    tally(
        &mut report,
        "fréquence cardiaque",
        detectors::heart_rate_anomaly::detect(pool).await,
    );
    tally(
        &mut report,
        "activité",
        detectors::activity_delta::detect(pool).await,
    );
    tally(
        &mut report,
        "tension",
        detectors::hypertension::detect(pool).await,
    );
    report
}

fn tally<E: std::fmt::Display>(report: &mut InsightReport, name: &str, outcome: Result<i64, E>) {
    match outcome {
        Ok(n) => report.generated += n,
        Err(e) => {
            log::warn!("[insights] analyse {name} en echec : {e}");
            report.failed.push(name.to_string());
        }
    }
}
