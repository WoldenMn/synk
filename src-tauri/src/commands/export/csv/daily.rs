use crate::error::AppError;
use crate::utils::dates::date_prefix;
use csv::Writer;
use sqlx::SqlitePool;

use crate::commands::export::shared::{fetch_activity, fetch_heart, fetch_sleep, fetch_workouts};

pub(super) async fn write_sleep(
    wtr: &mut Writer<Vec<u8>>,
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<u64, AppError> {
    wtr.write_record(["# sleep"])?;
    wtr.write_record(["date", "debut", "fin", "duree_totale_s", "score_sommeil"])?;

    let rows = fetch_sleep(pool, date_from, date_before).await?;

    let count = rows.len() as u64;
    for r in rows {
        wtr.write_record([
            &r.date,
            &r.start_at.unwrap_or_default(),
            &r.end_at.unwrap_or_default(),
            &r.duration_total.map(|v| v.to_string()).unwrap_or_default(),
            &r.sleep_score.map(|v| v.to_string()).unwrap_or_default(),
        ])?;
    }
    wtr.write_record([""])?;
    Ok(count)
}

pub(super) async fn write_activity(
    wtr: &mut Writer<Vec<u8>>,
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<u64, AppError> {
    wtr.write_record(["# activity"])?;
    wtr.write_record(["date", "pas", "distance_m", "calories_actives"])?;

    let rows = fetch_activity(pool, date_from, date_before).await?;

    let count = rows.len() as u64;
    for r in rows {
        wtr.write_record([
            &r.date,
            &r.steps.map(|v| v.to_string()).unwrap_or_default(),
            &r.distance.map(|v| format!("{:.0}", v)).unwrap_or_default(),
            &r.calories_active
                .map(|v| format!("{:.0}", v))
                .unwrap_or_default(),
        ])?;
    }
    wtr.write_record([""])?;
    Ok(count)
}

pub(super) async fn write_workouts(
    wtr: &mut Writer<Vec<u8>>,
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<u64, AppError> {
    wtr.write_record(["# workouts"])?;
    wtr.write_record([
        "date",
        "type",
        "duree_s",
        "distance_m",
        "calories",
        "fc_moy",
    ])?;

    let rows = fetch_workouts(pool, date_from, date_before).await?;

    let count = rows.len() as u64;
    for r in rows {
        wtr.write_record([
            &r.date,
            &super::text_cell(&r.category_label.unwrap_or_default()).into_owned(),
            &r.duration.map(|v| v.to_string()).unwrap_or_default(),
            &r.distance.map(|v| format!("{:.0}", v)).unwrap_or_default(),
            &r.calories.map(|v| format!("{:.0}", v)).unwrap_or_default(),
            &r.hr_average.map(|v| v.to_string()).unwrap_or_default(),
        ])?;
    }
    wtr.write_record([""])?;
    Ok(count)
}

pub(super) async fn write_heart(
    wtr: &mut Writer<Vec<u8>>,
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<u64, AppError> {
    wtr.write_record(["# heart"])?;
    wtr.write_record(["timestamp", "ecg_status", "frequence_cardiaque"])?;

    let rows = fetch_heart(pool, date_from, date_before).await?;

    let status_label = |s: Option<i64>| match s {
        Some(0) => "Normal".to_string(),
        Some(1) => "AFib".to_string(),
        Some(2) => "Inconclusif".to_string(),
        Some(3) => "Trop court".to_string(),
        _ => "-".to_string(),
    };

    let count = rows.len() as u64;
    for r in rows {
        wtr.write_record([
            date_prefix(&r.timestamp),
            &status_label(r.ecg_status),
            &r.heart_rate.map(|v| v.to_string()).unwrap_or_default(),
        ])?;
    }
    wtr.write_record([""])?;
    Ok(count)
}
