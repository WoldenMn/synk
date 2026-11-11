use serde::Serialize;
use sqlx::SqlitePool;

use super::config::CsvConfig;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SavedMapping {
    pub id: i64,
    pub name: String,
    pub config: CsvConfig,
}

fn decode(id: i64, name: String, json: &str) -> Option<SavedMapping> {
    serde_json::from_str::<CsvConfig>(json)
        .ok()
        .map(|config| SavedMapping { id, name, config })
}

pub async fn save(
    pool: &SqlitePool,
    name: &str,
    signature: &str,
    config: &CsvConfig,
) -> Result<i64, AppError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(AppError::Validation(
            "le nom d'une correspondance fait entre 1 et 80 caractères".into(),
        ));
    }
    config.validate_shape()?;
    let json = serde_json::to_string(config)
        .map_err(|e| AppError::Internal(format!("correspondance illisible : {e}")))?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO csv_mappings (name, signature, config) VALUES (?1, ?2, ?3) \
         ON CONFLICT(name) DO UPDATE SET signature = excluded.signature, \
         config = excluded.config, updated_at = datetime('now') RETURNING id",
    )
    .bind(name)
    .bind(signature)
    .bind(json)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub struct Matches {
    pub found: Vec<SavedMapping>,
    pub unreadable: usize,
}

pub async fn matching(pool: &SqlitePool, signature: &str) -> Result<Vec<SavedMapping>, AppError> {
    Ok(matching_report(pool, signature).await?.found)
}

pub async fn unreadable_count(pool: &SqlitePool, signature: &str) -> Result<usize, AppError> {
    Ok(matching_report(pool, signature).await?.unreadable)
}

pub async fn matching_report(pool: &SqlitePool, signature: &str) -> Result<Matches, AppError> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, name, config FROM csv_mappings WHERE signature = ?1 ORDER BY updated_at DESC, id DESC",
    )
    .bind(signature)
    .fetch_all(pool)
    .await?;
    let total = rows.len();
    let found: Vec<SavedMapping> = rows
        .into_iter()
        .filter_map(|(id, name, json)| decode(id, name, &json))
        .collect();
    let unreadable = total - found.len();
    if unreadable > 0 {
        log::warn!("[csv] {unreadable} correspondance(s) enregistree(s) illisible(s) ignoree(s)");
    }
    Ok(Matches { found, unreadable })
}

pub async fn remove(pool: &SqlitePool, id: i64) -> Result<bool, AppError> {
    let done = sqlx::query("DELETE FROM csv_mappings WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(done.rows_affected() == 1)
}
