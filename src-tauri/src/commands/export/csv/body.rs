use crate::error::AppError;
use crate::utils::dates::date_prefix;
use csv::Writer;
use sqlx::SqlitePool;

use crate::commands::export::shared::{
    fetch_blood_pressure, fetch_body_measures, fetch_measurements,
};

pub(super) async fn write_body_measures(
    wtr: &mut Writer<Vec<u8>>,
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<u64, AppError> {
    wtr.write_record(["# body_measures"])?;
    wtr.write_record([
        "date",
        "poids_kg",
        "masse_grasse_%",
        "masse_grasse_kg",
        "masse_maigre_kg",
        "masse_musculaire_kg",
        "masse_osseuse_kg",
        "hydratation_kg",
        "imc",
    ])?;

    let rows = fetch_body_measures(pool, date_from, date_before).await?;

    let count = rows.len() as u64;
    for r in rows {
        wtr.write_record([
            date_prefix(&r.measured_at),
            &r.weight.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            &r.fat_ratio.map(|v| format!("{:.1}", v)).unwrap_or_default(),
            &r.fat_mass.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            &r.fat_free_mass
                .map(|v| format!("{:.2}", v))
                .unwrap_or_default(),
            &r.muscle_mass
                .map(|v| format!("{:.2}", v))
                .unwrap_or_default(),
            &r.bone_mass.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            &r.hydration.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            &r.bmi.map(|v| format!("{:.1}", v)).unwrap_or_default(),
        ])?;
    }
    wtr.write_record([""])?;
    Ok(count)
}

pub(super) async fn write_blood_pressure(
    wtr: &mut Writer<Vec<u8>>,
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<u64, AppError> {
    wtr.write_record(["# blood_pressure"])?;
    wtr.write_record([
        "date",
        "systolique_mmhg",
        "diastolique_mmhg",
        "frequence_cardiaque_bpm",
    ])?;

    let rows = fetch_blood_pressure(pool, date_from, date_before).await?;

    let count = rows.len() as u64;
    for r in rows {
        wtr.write_record([
            date_prefix(&r.measured_at),
            &r.systolic.to_string(),
            &r.diastolic.to_string(),
            &r.heart_rate.map(|v| v.to_string()).unwrap_or_default(),
        ])?;
    }
    wtr.write_record([""])?;
    Ok(count)
}

pub(super) async fn write_measurements(
    wtr: &mut Writer<Vec<u8>>,
    pool: &SqlitePool,
    date_from: &str,
    date_before: &str,
) -> Result<u64, AppError> {
    wtr.write_record(["# measurements"])?;
    wtr.write_record(["date", "type", "valeur", "unite"])?;

    let rows = fetch_measurements(pool, date_from, date_before).await?;

    let count = rows.len() as u64;
    for r in rows {
        wtr.write_record([
            date_prefix(&r.measured_at),
            &super::text_cell(&r.label),
            &format!("{:.1}", r.value),
            &super::text_cell(&r.unit),
        ])?;
    }
    wtr.write_record([""])?;
    Ok(count)
}
