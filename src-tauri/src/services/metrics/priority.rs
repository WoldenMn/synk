use sqlx::SqlitePool;

use crate::error::AppError;

async fn definition_id(pool: &SqlitePool, slug: &str) -> Result<i64, AppError> {
    sqlx::query_scalar("SELECT id FROM metric_definitions WHERE slug = ?1")
        .bind(slug)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Validation(format!("metrique inconnue : {slug}")))
}

pub async fn get_priority(pool: &SqlitePool, slug: &str) -> Result<Vec<String>, AppError> {
    let id = definition_id(pool, slug).await?;
    Ok(priority_of(pool, id).await?)
}

pub(super) async fn priority_of(
    pool: &SqlitePool,
    definition_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT source_id FROM metric_source_priority WHERE definition_id = ?1 ORDER BY rank",
    )
    .bind(definition_id)
    .fetch_all(pool)
    .await
}

pub async fn set_priority(
    pool: &SqlitePool,
    slug: &str,
    ordered_sources: &[&str],
) -> Result<(), AppError> {
    let id = definition_id(pool, slug).await?;
    let mut seen: Vec<&str> = Vec::new();
    for source in ordered_sources {
        let known: Option<i64> = sqlx::query_scalar("SELECT 1 FROM sources WHERE id = ?1")
            .bind(source)
            .fetch_optional(pool)
            .await?;
        if known.is_none() {
            return Err(AppError::Validation(format!("source inconnue : {source}")));
        }
        if seen.contains(source) {
            return Err(AppError::Validation(format!(
                "source en double dans la priorite : {source}"
            )));
        }
        seen.push(source);
    }
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM metric_source_priority WHERE definition_id = ?1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for (rank, source) in ordered_sources.iter().enumerate() {
        sqlx::query(
            "INSERT INTO metric_source_priority (definition_id, source_id, rank) VALUES (?1, ?2, ?3)",
        )
        .bind(id)
        .bind(source)
        .bind(rank as i64)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
