use crate::error::AppError;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct DbMetrics {
    pub db_size_bytes: u64,
    pub audit_log_count: i64,
    pub body_measures_count: i64,
    pub blood_pressure_count: i64,
    pub sleep_count: i64,
    pub activity_count: i64,
    pub workouts_count: i64,
}

async fn count_table(pool: &SqlitePool, table: &'static str) -> Result<i64, AppError> {
    let sql = format!("SELECT COUNT(*) FROM {}", table);
    let count: (i64,) = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .fetch_one(pool)
        .await?;
    Ok(count.0)
}

pub(crate) async fn get_db_metrics_inner(
    pool: &SqlitePool,
    db_path: &std::path::Path,
) -> Result<DbMetrics, AppError> {
    let db_size_bytes = tokio::fs::metadata(db_path)
        .await
        .map_err(|e| AppError::Internal(format!("taille de la base illisible : {e}")))?
        .len();
    let audit_log_count = count_table(pool, "audit_log").await?;
    let body_measures_count = count_table(pool, "body_measures").await?;
    let blood_pressure_count = count_table(pool, "blood_pressure").await?;
    let sleep_count = count_table(pool, "sleep_records").await?;
    let activity_count = count_table(pool, "activity").await?;
    let workouts_count = count_table(pool, "workouts").await?;
    Ok(DbMetrics {
        db_size_bytes,
        audit_log_count,
        body_measures_count,
        blood_pressure_count,
        sleep_count,
        activity_count,
        workouts_count,
    })
}

#[tauri::command]
pub async fn get_db_metrics(
    pool: State<'_, SqlitePool>,
    app: tauri::AppHandle,
) -> Result<DbMetrics, AppError> {
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("app_data_dir: {}", e)))?;
    let db_path = dir.join("synk.db");
    get_db_metrics_inner(pool.inner(), &db_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_database_file_is_an_error_not_zero_bytes() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        crate::db::bootstrap_schema(&pool).await.unwrap();
        let tmp = tempfile::TempDir::new().unwrap();

        assert!(get_db_metrics_inner(&pool, &tmp.path().join("absente.db"))
            .await
            .is_err());
    }
}
