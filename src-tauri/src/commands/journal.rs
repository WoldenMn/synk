use crate::error::AppError;
use crate::validation::{check_date, check_text, check_text_opt};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct JournalEntry {
    pub id: i64,
    pub date: String,
    pub content: String,
    pub tags: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct JournalInput {
    pub date: String,
    pub content: String,
    pub tags: Option<String>,
}

impl JournalInput {
    pub fn validate(&self) -> Result<(), AppError> {
        check_date("Date", &self.date)?;
        check_text("Contenu", &self.content, 10_000)?;
        check_text_opt("Tags", self.tags.as_deref(), 200)
    }
}

#[tauri::command]
pub async fn get_journal_entries(
    pool: State<'_, SqlitePool>,
    from: Option<String>,
    to: Option<String>,
) -> Result<Vec<JournalEntry>, AppError> {
    let from_d = from.unwrap_or_else(|| "1970-01-01".into());
    let to_d = to.unwrap_or_else(|| "2999-12-31".into());
    sqlx::query_as::<_, JournalEntry>(
        "SELECT id, date, content, tags, created_at, updated_at FROM journal_entries \
         WHERE date >= ?1 AND date <= ?2 ORDER BY date DESC",
    )
    .bind(from_d)
    .bind(to_d)
    .fetch_all(pool.inner())
    .await
    .map_err(AppError::from)
}

#[tauri::command]
pub async fn upsert_journal_entry(
    pool: State<'_, SqlitePool>,
    data: JournalInput,
) -> Result<i64, AppError> {
    upsert_journal_entry_inner(pool.inner(), data).await
}

pub async fn upsert_journal_entry_inner(
    pool: &SqlitePool,
    data: JournalInput,
) -> Result<i64, AppError> {
    data.validate()?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (date, content, tags, updated_at) VALUES (?1, ?2, ?3, datetime('now')) \
         ON CONFLICT(date) DO UPDATE SET content = excluded.content, tags = excluded.tags, updated_at = datetime('now') RETURNING id",
    )
    .bind(&data.date)
    .bind(&data.content)
    .bind(&data.tags)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

#[tauri::command]
pub async fn delete_journal_entry(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    crate::commands::crud::delete_by_id(pool.inner(), "journal_entries", "journal_entries", id)
        .await
}
