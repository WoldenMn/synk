use crate::error::AppError;
use crate::utils::text::truncate_chars;
use crate::validation::check_limit;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ErrorLogEntry {
    pub id: i64,
    pub occurred_at: String,
    pub source: String,
    pub code: String,
    pub message: String,
    pub context: Option<String>,
}

const MESSAGE_MAX_CHARS: usize = 2000;
const CONTEXT_MAX_CHARS: usize = 5000;

pub async fn log_error(
    pool: &SqlitePool,
    source: &str,
    code: &str,
    message: &str,
    context: Option<&str>,
) -> Result<(), AppError> {
    if source.trim().is_empty() || code.trim().is_empty() {
        return Err(AppError::Validation("source et code requis".into()));
    }
    let message = truncate_chars(message, MESSAGE_MAX_CHARS);
    let context = context.map(|c| truncate_chars(c, CONTEXT_MAX_CHARS));
    sqlx::query("INSERT INTO error_log (source, code, message, context) VALUES (?1, ?2, ?3, ?4)")
        .bind(source)
        .bind(code)
        .bind(&message)
        .bind(context.as_deref())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_error_log(
    pool: &SqlitePool,
    limit: i64,
    code_filter: Option<&str>,
) -> Result<Vec<ErrorLogEntry>, AppError> {
    let limit = check_limit(Some(limit), 200)?;
    let rows = sqlx::query_as::<_, ErrorLogEntry>(
        "SELECT id, occurred_at, source, code, message, context FROM error_log \
         WHERE (?1 IS NULL OR code = ?1) ORDER BY occurred_at DESC, id DESC LIMIT ?2",
    )
    .bind(code_filter)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn purge_error_log(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::query("DELETE FROM error_log").execute(pool).await?;
    Ok(())
}

/// Best-effort : conserve les `cap` entrees les plus recentes.
pub async fn auto_purge(pool: &SqlitePool, cap: i64) -> Result<(), AppError> {
    sqlx::query(
        "DELETE FROM error_log WHERE id NOT IN \
         (SELECT id FROM error_log ORDER BY occurred_at DESC, id DESC LIMIT ?1)",
    )
    .bind(cap)
    .execute(pool)
    .await?;
    Ok(())
}
