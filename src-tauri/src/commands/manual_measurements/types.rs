use crate::error::AppError;
use crate::validation::{check_text, check_text_max};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MeasurementType {
    pub id: i64,
    pub slug: String,
    pub label: String,
    pub unit: String,
    pub category: String,
    pub sort_order: i64,
    pub is_builtin: i64,
    pub target_value: Option<f64>,
    pub target_direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMeasurementTypeInput {
    pub slug: String,
    pub label: String,
    pub unit: String,
}

impl AddMeasurementTypeInput {
    pub fn validate(&self) -> Result<(), AppError> {
        let slug_ok = !self.slug.is_empty()
            && self.slug.chars().count() <= 60
            && self
                .slug
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !slug_ok {
            return Err(AppError::Validation(
                "Identifiant : 1 a 60 caracteres parmi a-z, 0-9 et _".into(),
            ));
        }
        check_text("Nom", &self.label, 60)?;
        check_text_max("Unite", &self.unit, 20)
    }
}

#[tauri::command]
pub async fn get_measurement_types(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<MeasurementType>, AppError> {
    sqlx::query_as::<_, MeasurementType>(
        "SELECT id, slug, label, unit, category, sort_order, is_builtin, target_value, target_direction \
         FROM measurement_types ORDER BY sort_order ASC",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(AppError::from)
}

#[tauri::command]
pub async fn add_measurement_type(
    pool: State<'_, SqlitePool>,
    data: AddMeasurementTypeInput,
) -> Result<i64, AppError> {
    data.validate()?;
    let max_order: Option<i64> =
        sqlx::query_scalar("SELECT MAX(sort_order) FROM measurement_types")
            .fetch_one(pool.inner())
            .await?;
    let next_order = max_order.unwrap_or(0) + 1;

    // INSERT cible la table reelle (les vues de retrocompat ne sont pas updatables sans trigger).
    let result = sqlx::query(
        "INSERT INTO measurement_types_legacy (slug, label, unit, category, sort_order, is_builtin) \
         VALUES (?1, ?2, ?3, 'custom', ?4, 0)",
    )
    .bind(&data.slug)
    .bind(&data.label)
    .bind(&data.unit)
    .bind(next_order)
    .execute(pool.inner())
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn delete_measurement_type_inner(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let deleted =
        sqlx::query("DELETE FROM measurement_types_legacy WHERE id = ?1 AND is_builtin = 0")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();
    if deleted == 0 {
        let builtin: Option<i64> =
            sqlx::query_scalar("SELECT is_builtin FROM measurement_types_legacy WHERE id = ?1")
                .bind(id)
                .fetch_optional(pool)
                .await?;
        return Err(match builtin {
            Some(_) => AppError::Forbidden(format!("type de mesure #{id} integre a Synk")),
            None => AppError::NotFound {
                entity: "measurement_types",
                id,
            },
        });
    }
    crate::services::audit::record_audit_or_warn(
        pool,
        "delete",
        "measurement_types",
        Some(id),
        None,
    )
    .await;
    Ok(())
}

#[tauri::command]
pub async fn delete_measurement_type(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    delete_measurement_type_inner(pool.inner(), id).await
}
