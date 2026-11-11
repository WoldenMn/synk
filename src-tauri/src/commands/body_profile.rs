use crate::body_catalog;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct SessionEntry {
    pub slug: String,
    pub value: f64,
}

// `pub` requis : les tests d'integration dans `tests/` sont compiles comme
// des crates separes, donc `pub(crate)` ne suffirait pas.
pub async fn add_measurement_session_inner(
    pool: &SqlitePool,
    measured_at: String,
    entries: Vec<SessionEntry>,
) -> Result<i64, AppError> {
    if entries.is_empty() {
        return Err(AppError::Validation(
            "Un relevé doit contenir au moins une mesure".into(),
        ));
    }
    if measured_at.trim().is_empty() {
        return Err(AppError::Validation("Date de releve manquante".into()));
    }

    // Validation complete AVANT d'ouvrir la transaction : une seule valeur
    // invalide doit faire echouer le releve entier sans rien ecrire.
    for entry in &entries {
        let def = body_catalog::find(&entry.slug).ok_or_else(|| {
            AppError::Validation(format!("Mensuration inconnue : {}", entry.slug))
        })?;
        crate::validation::check_f64(&def.label, entry.value, def.min, def.max)?;
    }

    let mut tx = pool.begin().await?;
    let mut written = 0i64;
    for entry in &entries {
        let type_id: i64 = sqlx::query_scalar("SELECT id FROM measurement_types WHERE slug = ?1")
            .bind(&entry.slug)
            .fetch_one(&mut *tx)
            .await?;

        sqlx::query(
            "INSERT INTO manual_measurements_legacy \
             (measurement_type_id, measured_at, value, notes) \
             VALUES (?1, ?2, ?3, NULL)",
        )
        .bind(type_id)
        .bind(&measured_at)
        .bind(entry.value)
        .execute(&mut *tx)
        .await?;
        written += 1;
    }
    tx.commit().await?;
    Ok(written)
}

#[tauri::command]
pub async fn add_measurement_session(
    pool: State<'_, SqlitePool>,
    measured_at: String,
    entries: Vec<SessionEntry>,
) -> Result<i64, AppError> {
    add_measurement_session_inner(pool.inner(), measured_at, entries).await
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SheetRow {
    pub slug: String,
    pub label: String,
    pub unit: String,
    pub value: Option<f64>,
    pub measured_at: Option<String>,
}

pub async fn get_body_sheet_inner(pool: &SqlitePool) -> Result<Vec<SheetRow>, AppError> {
    let rows = sqlx::query_as::<_, SheetRow>(
        "SELECT mt.slug AS slug, mt.label AS label, mt.unit AS unit, \
                mm.value AS value, mm.measured_at AS measured_at \
         FROM measurement_types mt \
         LEFT JOIN manual_measurements mm ON mm.id = ( \
             SELECT id FROM manual_measurements \
             WHERE measurement_type_id = mt.id \
             ORDER BY measured_at DESC, id DESC LIMIT 1 \
         ) \
         WHERE mt.is_builtin = 1 \
         ORDER BY mt.sort_order ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[tauri::command]
pub async fn get_body_sheet(pool: State<'_, SqlitePool>) -> Result<Vec<SheetRow>, AppError> {
    get_body_sheet_inner(pool.inner()).await
}
