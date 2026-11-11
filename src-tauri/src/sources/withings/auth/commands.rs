use sqlx::SqlitePool;

use crate::error::AppError;
use crate::sources::store;
use crate::sources::withings::spec::OAUTH;
use crate::sources::SourceId;

pub fn authorization_url(client_id: &str, state: &str) -> String {
    OAUTH.authorization_url(client_id, state, None)
}

pub async fn is_connected_inner(pool: &SqlitePool) -> Result<bool, AppError> {
    store::is_connected(pool, SourceId::WITHINGS).await
}

pub async fn disconnect_inner(pool: &SqlitePool) -> Result<(), AppError> {
    store::delete_auth(pool, SourceId::WITHINGS).await?;

    store::reset_cursors(pool, SourceId::WITHINGS).await?;

    Ok(())
}
