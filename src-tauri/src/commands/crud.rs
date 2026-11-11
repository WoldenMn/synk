use crate::error::AppError;
use crate::services::audit;
use crate::utils::dates::day_range;
use sqlx::SqlitePool;

pub trait CrudTable: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin {
    const TABLE: &'static str;
    const SELECT_COLS: &'static str;
    const DATE_COL: &'static str;
    const RANGE_ORDER: &'static str = "ASC";
}

pub async fn select_range<T: CrudTable>(
    pool: &SqlitePool,
    from: String,
    to: String,
) -> Result<Vec<T>, AppError> {
    let range = day_range(&from, &to)?;
    let (from_d, to_d) = (range.from, range.before);
    let sql = format!(
        "SELECT {} FROM {} WHERE {} >= ?1 AND {} < ?2 ORDER BY {} {}",
        T::SELECT_COLS,
        T::TABLE,
        T::DATE_COL,
        T::DATE_COL,
        T::DATE_COL,
        T::RANGE_ORDER,
    );
    let rows = sqlx::query_as::<_, T>(sqlx::AssertSqlSafe(sql))
        .bind(&from_d)
        .bind(&to_d)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn select_latest<T: CrudTable>(pool: &SqlitePool) -> Result<Option<T>, AppError> {
    let sql = format!(
        "SELECT {} FROM {} ORDER BY {} DESC LIMIT 1",
        T::SELECT_COLS,
        T::TABLE,
        T::DATE_COL,
    );
    let row = sqlx::query_as::<_, T>(sqlx::AssertSqlSafe(sql))
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn select_at<T: CrudTable>(
    pool: &SqlitePool,
    at: &str,
    source: &str,
) -> Result<Option<T>, AppError> {
    let sql = format!(
        "SELECT {} FROM {} WHERE {} = ?1 AND source = ?2 ORDER BY id DESC LIMIT 1",
        T::SELECT_COLS,
        T::TABLE,
        T::DATE_COL,
    );
    let row = sqlx::query_as::<_, T>(sqlx::AssertSqlSafe(sql))
        .bind(at)
        .bind(source)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn delete_with_audit(
    pool: &SqlitePool,
    table: &'static str,
    id: i64,
) -> Result<(), AppError> {
    delete_manual(pool, table, table, id).await
}

pub async fn delete_manual(
    pool: &SqlitePool,
    table: &'static str,
    entity: &'static str,
    id: i64,
) -> Result<(), AppError> {
    let sql = format!("DELETE FROM {table} WHERE id = ?1 AND source = 'manual'");
    if sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected()
        == 0
    {
        let exists = format!("SELECT COUNT(*) FROM {table} WHERE id = ?1");
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(exists))
            .bind(id)
            .fetch_one(pool)
            .await?;
        return Err(if n > 0 {
            AppError::Forbidden(format!(
                "{entity} #{id} : donnee synchronisee depuis Withings"
            ))
        } else {
            AppError::NotFound { entity, id }
        });
    }
    audit::record_audit_or_warn(pool, "delete", entity, Some(id), None).await;
    Ok(())
}

pub async fn delete_by_id(
    pool: &SqlitePool,
    table: &'static str,
    entity: &'static str,
    id: i64,
) -> Result<(), AppError> {
    let sql = format!("DELETE FROM {table} WHERE id = ?1");
    if sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected()
        == 0
    {
        return Err(AppError::NotFound { entity, id });
    }
    audit::record_audit_or_warn(pool, "delete", entity, Some(id), None).await;
    Ok(())
}

pub async fn record_audit_insert(pool: &SqlitePool, table: &str, id: i64) {
    audit::record_audit_or_warn(pool, "insert", table, Some(id), None).await;
}
