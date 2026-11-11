use sqlx::SqlitePool;

use crate::error::AppError;
use crate::sources::store;
use crate::sources::SourceId;

pub async fn get_last_sync(pool: &SqlitePool, data_type: &str) -> Result<i64, AppError> {
    store::get_last_sync(pool, SourceId::WITHINGS, data_type).await
}

/// Persiste le curseur de sync. Cluster B (#1/#10) : le curseur DOIT etre
/// capture `now()` AVANT le fetch reseau par le caller, jamais recalcule ici
/// (sinon les enregistrements crees pendant la sync tombent dans une fenetre
/// aveugle au cycle suivant).
pub async fn update_last_sync(
    pool: &SqlitePool,
    data_type: &str,
    cursor: i64,
) -> Result<(), AppError> {
    store::set_last_sync(pool, SourceId::WITHINGS, data_type, cursor).await
}
