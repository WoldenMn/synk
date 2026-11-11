use sqlx::SqlitePool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::AppHandle;

use super::flow::{generate_state, OAuthFlow};
use super::pkce::Verifier;
use super::reauth::REFRESH_EXPIRY;
use super::server::spawn_callback_server;
use super::spec::OAuthSpec;
use super::token::exchange_code;
use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::sources::store;

pub async fn start_authorization(
    spec: &'static OAuthSpec,
    pool: &SqlitePool,
    vault: &Arc<CryptoVault>,
    app: AppHandle,
) -> Result<String, AppError> {
    let client_id = store::require_client_id(pool, vault.as_ref(), spec.id).await?;

    let state = generate_state()?;
    let verifier = if spec.pkce {
        Some(Verifier::generate()?)
    } else {
        None
    };
    let challenge = verifier.as_ref().map(Verifier::challenge);
    let url = spec.authorization_url(&client_id, &state, challenge.as_deref());

    let flow = OAuthFlow::global();
    flow.set_pending(spec.id, state, verifier);

    if flow
        .server_started
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        spawn_callback_server(spec.id, pool.clone(), vault.clone(), app);
    }

    Ok(url)
}

pub async fn exchange_and_store(
    spec: &OAuthSpec,
    base: &str,
    pool: &SqlitePool,
    vault: &CryptoVault,
    code: &str,
    verifier: Option<&Verifier>,
) -> Result<(), AppError> {
    let (client_id, client_secret) = store::require_credentials(pool, vault, spec.id).await?;
    let tokens = exchange_code(spec, base, &client_id, &client_secret, code, verifier).await?;
    store::save_auth(pool, vault, spec.id, &tokens)
        .await
        .map_err(|e| AppError::Internal(format!("save tokens: {e}")))?;
    if let Some(lifetime) = spec.refresh_lifetime_secs {
        let expires_at = chrono::Utc::now().timestamp() + lifetime;
        store::set_last_sync(pool, spec.id, REFRESH_EXPIRY, expires_at).await?;
    }
    Ok(())
}
