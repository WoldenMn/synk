pub mod records;
pub mod types;

use crate::body_catalog;
use crate::error::AppError;
use sqlx::SqlitePool;
use tauri::State;

use self::types::{Body, Health, KeyMeasurement, ProfileCard, SizeEntry};

async fn fetch_profile_row(
    pool: &SqlitePool,
) -> Result<Option<(Option<i64>, Option<String>, Option<f64>)>, sqlx::Error> {
    sqlx::query_as("SELECT age, sex, height_cm FROM user_profile WHERE id = 1")
        .fetch_optional(pool)
        .await
}

async fn fetch_body(
    pool: &SqlitePool,
) -> Result<
    Option<(
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        String,
    )>,
    sqlx::Error,
> {
    sqlx::query_as(
        "SELECT weight, bmi, fat_ratio, muscle_mass, hydration, measured_at \
         FROM body_measures ORDER BY measured_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}

async fn fetch_key_measurements(pool: &SqlitePool) -> Result<Vec<KeyMeasurement>, sqlx::Error> {
    let mut out = Vec::new();
    for def in body_catalog::catalog().iter().filter(|m| m.is_key) {
        let row: Option<(f64, String)> = sqlx::query_as(
            "SELECT mm.value, mm.measured_at FROM manual_measurements mm \
             JOIN measurement_types mt ON mt.id = mm.measurement_type_id \
             WHERE mt.slug = ?1 ORDER BY mm.measured_at DESC, mm.id DESC LIMIT 1",
        )
        .bind(&def.slug)
        .fetch_optional(pool)
        .await?;
        out.push(KeyMeasurement {
            slug: def.slug.clone(),
            label: def.label.clone(),
            value: row.as_ref().map(|(v, _)| *v),
            measured_at: row.map(|(_, d)| d),
        });
    }
    Ok(out)
}

async fn fetch_active_treatments(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT name FROM medications WHERE active = 1 ORDER BY name ASC")
        .fetch_all(pool)
        .await
}

async fn fetch_sizes(pool: &SqlitePool) -> Result<Vec<SizeEntry>, sqlx::Error> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT category, value FROM clothing_sizes ORDER BY category ASC")
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .map(|(category, value)| SizeEntry { category, value })
        .collect())
}

// `pub` requis : les tests d'integration dans `tests/` sont compiles comme
// des crates separes.
pub async fn get_profile_card_inner(pool: &SqlitePool) -> Result<ProfileCard, AppError> {
    let (profile, body_row, key_measurements, treatments, sizes) = tokio::try_join!(
        fetch_profile_row(pool),
        fetch_body(pool),
        fetch_key_measurements(pool),
        fetch_active_treatments(pool),
        fetch_sizes(pool),
    )?;
    let records = records::fetch_records(pool).await?;

    let (age, sex, height_cm) = profile.unwrap_or_default();

    let body = match body_row {
        Some((weight, bmi, fat_ratio, muscle_mass, hydration, measured_at)) => Body {
            age,
            sex,
            height_cm,
            weight_kg: weight,
            bmi,
            fat_ratio,
            muscle_mass,
            hydration,
            measured_at: Some(measured_at),
        },
        None => Body {
            age,
            sex,
            height_cm,
            ..Default::default()
        },
    };

    let health = Health {
        active_treatments: treatments,
    };

    Ok(ProfileCard {
        body,
        key_measurements,
        health,
        sizes,
        records,
    })
}

#[tauri::command]
pub async fn get_profile_card(pool: State<'_, SqlitePool>) -> Result<ProfileCard, AppError> {
    get_profile_card_inner(pool.inner()).await
}
