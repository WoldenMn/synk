use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct ClothingInputs {
    pub chest: Option<f64>,
    pub neck: Option<f64>,
    pub waist: Option<f64>,
    pub inseam: Option<f64>,
}

async fn latest_by_slug(pool: &SqlitePool, slug: &str) -> Result<Option<f64>, sqlx::Error> {
    sqlx::query_scalar::<_, f64>(
        "SELECT mm.value FROM manual_measurements mm \
         JOIN measurement_types mt ON mt.id = mm.measurement_type_id \
         WHERE mt.slug = ?1 ORDER BY mm.measured_at DESC, mm.id DESC LIMIT 1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn get_clothing_inputs_inner(pool: &SqlitePool) -> Result<ClothingInputs, AppError> {
    Ok(ClothingInputs {
        chest: latest_by_slug(pool, "chest").await?,
        neck: latest_by_slug(pool, "neck").await?,
        waist: latest_by_slug(pool, "waist").await?,
        inseam: latest_by_slug(pool, "inseam").await?,
    })
}

#[tauri::command]
pub async fn get_clothing_inputs(pool: State<'_, SqlitePool>) -> Result<ClothingInputs, AppError> {
    get_clothing_inputs_inner(pool.inner()).await
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ClothingSize {
    pub category: String,
    pub value: String,
    pub source: String,
    pub updated_at: String,
}

pub async fn get_clothing_sizes_inner(pool: &SqlitePool) -> Result<Vec<ClothingSize>, AppError> {
    let rows = sqlx::query_as::<_, ClothingSize>(
        "SELECT category, value, source, updated_at FROM clothing_sizes ORDER BY category",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn set_clothing_size_inner(
    pool: &SqlitePool,
    category: &str,
    value: &str,
    source: &str,
) -> Result<(), AppError> {
    if source != "override" && source != "manual" {
        return Err(AppError::Validation(format!("source invalide: {source}")));
    }
    if category.trim().is_empty() || value.trim().is_empty() {
        return Err(AppError::Validation("category et value requis".into()));
    }
    sqlx::query(
        "INSERT INTO clothing_sizes (category, value, source, updated_at) \
         VALUES (?1, ?2, ?3, datetime('now')) \
         ON CONFLICT(category) DO UPDATE SET \
           value = excluded.value, source = excluded.source, updated_at = datetime('now')",
    )
    .bind(category)
    .bind(value)
    .bind(source)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_clothing_size_inner(pool: &SqlitePool, category: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM clothing_sizes WHERE category = ?1")
        .bind(category)
        .execute(pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_clothing_sizes(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<ClothingSize>, AppError> {
    get_clothing_sizes_inner(pool.inner()).await
}

#[tauri::command]
pub async fn set_clothing_size(
    pool: State<'_, SqlitePool>,
    category: String,
    value: String,
    source: String,
) -> Result<(), AppError> {
    set_clothing_size_inner(pool.inner(), &category, &value, &source).await
}

#[tauri::command]
pub async fn delete_clothing_size(
    pool: State<'_, SqlitePool>,
    category: String,
) -> Result<(), AppError> {
    delete_clothing_size_inner(pool.inner(), &category).await
}

#[derive(Debug, Deserialize)]
pub struct EstimatedSize {
    pub category: String,
    pub value: String,
}

// `pub` requis : les tests d'integration dans `tests/` sont compiles comme
// des crates separes.
pub async fn save_estimated_sizes_inner(
    pool: &SqlitePool,
    sizes: Vec<EstimatedSize>,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    for size in &sizes {
        // La clause WHERE protege les corrections de l'utilisateur : une
        // estimation ne remplace qu'une estimation precedente, jamais une
        // ligne 'manual' ou 'override'.
        sqlx::query(
            "INSERT INTO clothing_sizes (category, value, source, updated_at) \
             VALUES (?1, ?2, 'estimated', datetime('now')) \
             ON CONFLICT(category) DO UPDATE SET \
                 value = ?2, updated_at = datetime('now') \
             WHERE clothing_sizes.source = 'estimated'",
        )
        .bind(&size.category)
        .bind(&size.value)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

#[tauri::command]
pub async fn save_estimated_sizes(
    pool: State<'_, SqlitePool>,
    sizes: Vec<EstimatedSize>,
) -> Result<(), AppError> {
    save_estimated_sizes_inner(pool.inner(), sizes).await
}
