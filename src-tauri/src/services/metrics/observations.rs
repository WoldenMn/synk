use std::collections::HashMap;

use sqlx::SqliteConnection;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::metric_model::Observation;
use crate::metric_repository::{classify, LegacyTable, MetricSource};
use crate::validation::check_f64;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WriteReport {
    pub inserted: usize,
    pub ignored: usize,
}

#[derive(Clone, Copy)]
enum Target {
    BodyColumn,
    VitalType,
    SleepScore,
    Steps,
    Metrics,
}

#[derive(Clone, Copy)]
struct Resolved {
    definition_id: i64,
    target: Target,
    min: f64,
    max: f64,
}

fn target_of(slug: &str, source: MetricSource) -> Result<Target, AppError> {
    let refused = |why: &str| {
        Err(AppError::Validation(format!(
            "{slug} : une observation seule ne peut pas etre ecrite ({why})"
        )))
    };
    match source {
        MetricSource::Legacy(LegacyTable::BodyMeasures) => Ok(Target::BodyColumn),
        MetricSource::Legacy(LegacyTable::Vitals) => Ok(Target::VitalType),
        MetricSource::Legacy(LegacyTable::SleepRecords) if slug == "sleep_score" => {
            Ok(Target::SleepScore)
        }
        MetricSource::Legacy(LegacyTable::Activity) if slug == "steps" => Ok(Target::Steps),
        MetricSource::Metrics | MetricSource::Split(_) => Ok(Target::Metrics),
        MetricSource::Legacy(LegacyTable::BloodPressure) => {
            refused("systolique et diastolique vont par paire")
        }
        MetricSource::Legacy(LegacyTable::ManualMeasurements) => {
            refused("les mensurations sont saisies a la main")
        }
        MetricSource::Legacy(_) => refused("pas de colonne pour cette metrique"),
    }
}

async fn resolve(conn: &mut SqliteConnection, slug: &str) -> Result<Resolved, AppError> {
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(AppError::Validation(format!("metrique invalide : {slug}")));
    }
    let row: Option<(i64, String, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT id, category, min_value, max_value FROM metric_definitions WHERE slug = ?1",
    )
    .bind(slug)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((definition_id, category, min, max)) = row else {
        return Err(AppError::Validation(format!("metrique inconnue : {slug}")));
    };
    let Some(source) = classify(&category, slug) else {
        return Err(AppError::Validation(format!("metrique inconnue : {slug}")));
    };
    Ok(Resolved {
        definition_id,
        target: target_of(slug, source)?,
        min: min.unwrap_or(f64::MIN),
        max: max.unwrap_or(f64::MAX),
    })
}

fn conflict(revise: bool, key: &str, column: &str) -> String {
    if revise {
        format!(
            " ON CONFLICT({key}) DO UPDATE SET {column} = excluded.{column} WHERE {column} IS NOT excluded.{column}"
        )
    } else {
        " ON CONFLICT DO NOTHING".to_string()
    }
}

async fn insert(
    conn: &mut SqliteConnection,
    resolved: &Resolved,
    o: &Observation,
    external_id: &str,
    revise: bool,
) -> Result<u64, AppError> {
    let at = o.measured_at.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let day = o.measured_at.format("%Y-%m-%d").to_string();
    let source = o.source.as_str();
    let query = match resolved.target {
        Target::BodyColumn => format!(
            "INSERT INTO body_measures_legacy (measured_at, source, {col}, external_id, device) VALUES (?1, ?2, ?3, ?4, ?6){}",
            conflict(revise, "source, external_id", &o.metric),
            col = o.metric
        ),
        Target::VitalType => format!(
            "INSERT INTO vitals_legacy (measured_at, source, value, external_id, vital_type, device) VALUES (?1, ?2, ?3, ?4, ?5, ?6){}",
            conflict(revise, "source, external_id", "value")
        ),
        Target::SleepScore => "INSERT INTO sleep_records (date, source, sleep_score, external_id, device) VALUES (?1, ?2, ?3, ?4, ?6) ON CONFLICT DO NOTHING".to_string(),
        Target::Steps => "INSERT INTO activity (date, source, steps, external_id, device) VALUES (?1, ?2, ?3, ?4, ?6) ON CONFLICT DO NOTHING".to_string(),
        Target::Metrics => format!(
            "INSERT INTO metrics (measured_at, source, value, external_id, definition_id, device) VALUES (?1, ?2, ?3, ?4, ?5, ?6){}",
            conflict(revise, "definition_id, measured_at, source, external_id", "value")
        ),
    };
    let day_keyed = matches!(resolved.target, Target::SleepScore | Target::Steps);
    let first = if day_keyed { day } else { at };
    let value = if day_keyed { o.value.round() } else { o.value };
    let fifth: Option<String> = match resolved.target {
        Target::VitalType => Some(o.metric.clone()),
        _ => None,
    };
    let definition = match resolved.target {
        Target::Metrics => Some(resolved.definition_id),
        _ => None,
    };
    let mut q = sqlx::query(sqlx::AssertSqlSafe(query))
        .bind(first)
        .bind(source)
        .bind(value)
        .bind(external_id);
    q = match (&fifth, definition) {
        (Some(text), _) => q.bind(text.clone()),
        (None, Some(id)) => q.bind(id),
        (None, None) => q.bind(Option::<String>::None),
    };
    Ok(q.bind(o.device.clone())
        .execute(&mut *conn)
        .await?
        .rows_affected())
}

#[derive(Default)]
pub struct ObservationWriter {
    cache: HashMap<String, Resolved>,
    revise: bool,
}

impl ObservationWriter {
    pub fn revising() -> Self {
        Self {
            cache: HashMap::new(),
            revise: true,
        }
    }

    pub async fn write(
        &mut self,
        conn: &mut SqliteConnection,
        o: &Observation,
    ) -> Result<bool, AppError> {
        let resolved = match self.cache.get(&o.metric) {
            Some(r) => *r,
            None => {
                let r = resolve(conn, &o.metric).await?;
                self.cache.insert(o.metric.clone(), r);
                r
            }
        };
        check_f64(&o.metric, o.value, resolved.min, resolved.max)?;
        let external_id = o.external_id.clone().unwrap_or_else(|| {
            format!(
                "{}@{}",
                o.metric,
                o.measured_at.format("%Y-%m-%dT%H:%M:%SZ")
            )
        });
        Ok(insert(conn, &resolved, o, &external_id, self.revise).await? == 1)
    }
}

#[derive(Debug, Default)]
pub struct TolerantReport {
    pub inserted: usize,
    pub notes: Vec<String>,
}

pub async fn write_tolerant(
    pool: &SqlitePool,
    observations: &[Observation],
) -> Result<TolerantReport, AppError> {
    let mut tx = pool.begin().await?;
    let mut writer = ObservationWriter::revising();
    let mut report = TolerantReport::default();
    for o in observations {
        match writer.write(&mut tx, o).await {
            Ok(true) => report.inserted += 1,
            Ok(false) => {}
            Err(AppError::Validation(why)) => report.notes.push(why),
            Err(other) => return Err(other),
        }
    }
    tx.commit().await?;
    Ok(report)
}

pub async fn write_observations(
    pool: &SqlitePool,
    observations: &[Observation],
) -> Result<WriteReport, AppError> {
    let mut report = WriteReport::default();
    let mut conn = pool.acquire().await?;
    let mut writer = ObservationWriter::revising();
    for o in observations {
        if writer.write(&mut conn, o).await? {
            report.inserted += 1;
        } else {
            report.ignored += 1;
        }
    }
    Ok(report)
}
