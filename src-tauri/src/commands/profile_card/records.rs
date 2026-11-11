use crate::error::AppError;
use sqlx::SqlitePool;

use super::types::Record;
use crate::metric_repository::MetricPoint;
use crate::services::metrics::select_series;

struct RecordSpec {
    key: &'static str,
    label: &'static str,
    unit: &'static str,
    sql: &'static str,
}

// ORDER BY ... LIMIT 1 et jamais MIN()/MAX() : une fonction d'agregat donne
// la valeur mais perd la date a laquelle elle a ete atteinte.
//
// Colonnes INTEGER : sqlx refuse le decodage direct en f64 (mismatched
// types REAL/INTEGER), donc lues en i64 puis converties.
const INTEGER_SPECS: &[RecordSpec] = &[
    // Durees en secondes, comme dans sleep_records et workouts (total_sleep_time
    // de Withings) : l'unite « s » le dit, l'affichage les convertit.
    RecordSpec {
        key: "sleep_duration_max",
        label: "Plus longue nuit",
        unit: "s",
        sql: "SELECT duration_total, date FROM sleep_records WHERE duration_total IS NOT NULL ORDER BY duration_total DESC LIMIT 1",
    },
    RecordSpec {
        key: "heart_rate_min",
        label: "Fréquence cardiaque la plus basse",
        unit: "bpm",
        sql: "SELECT heart_rate, timestamp FROM heart_records WHERE heart_rate IS NOT NULL ORDER BY heart_rate ASC LIMIT 1",
    },
    RecordSpec {
        key: "workout_duration_max",
        label: "Entraînement le plus long",
        unit: "s",
        sql: "SELECT duration, date FROM workouts WHERE duration IS NOT NULL ORDER BY duration DESC LIMIT 1",
    },
];

struct SeriesRecord {
    key: &'static str,
    label: &'static str,
    unit: &'static str,
    slug: &'static str,
    highest: bool,
}

// Les records d'une metrique lisent la meme selection que l'ecran
// (`select_series`) : un poids saisi deux fois par deux sources compte une fois.
const SERIES_RECORDS: &[SeriesRecord] = &[
    SeriesRecord {
        key: "weight_min",
        label: "Poids minimum",
        unit: "kg",
        slug: "weight",
        highest: false,
    },
    SeriesRecord {
        key: "weight_max",
        label: "Poids maximum",
        unit: "kg",
        slug: "weight",
        highest: true,
    },
    SeriesRecord {
        key: "steps_max",
        label: "Meilleur total de pas",
        unit: "pas",
        slug: "steps",
        highest: true,
    },
    SeriesRecord {
        key: "sleep_score_max",
        label: "Meilleur score de sommeil",
        unit: "/100",
        slug: "sleep_score",
        highest: true,
    },
];

async fn series_record(pool: &SqlitePool, spec: &SeriesRecord) -> Result<Option<Record>, AppError> {
    let points = select_series(pool, spec.slug, "1900-01-01", "2999-12-31").await?;
    let mut best: Option<&MetricPoint> = None;
    for p in &points {
        let better = best.is_none_or(|b| {
            if spec.highest {
                p.value > b.value
            } else {
                p.value < b.value
            }
        });
        if better {
            best = Some(p);
        }
    }
    Ok(best.map(|p| Record {
        key: spec.key.to_string(),
        label: spec.label.to_string(),
        value: p.value,
        unit: spec.unit.to_string(),
        achieved_at: p.measured_at.clone(),
    }))
}

pub async fn fetch_records(pool: &SqlitePool) -> Result<Vec<Record>, AppError> {
    let mut records = Vec::new();

    for spec in SERIES_RECORDS {
        if let Some(record) = series_record(pool, spec).await? {
            records.push(record);
        }
    }

    for spec in INTEGER_SPECS {
        let row: Option<(i64, String)> = sqlx::query_as(spec.sql).fetch_optional(pool).await?;
        if let Some((value, achieved_at)) = row {
            records.push(Record {
                key: spec.key.to_string(),
                label: spec.label.to_string(),
                value: value as f64,
                unit: spec.unit.to_string(),
                achieved_at,
            });
        }
    }

    Ok(records)
}
