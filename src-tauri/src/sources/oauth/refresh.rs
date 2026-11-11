use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use sqlx::SqlitePool;
use zeroize::Zeroizing;

use super::reauth::REFRESH_EXPIRY;
use super::spec::OAuthSpec;
use super::token::refresh_token;
use crate::crypto::CryptoVault;
use crate::error::AppError;
use crate::sources::{label_of, store, SourceId};

type SourceLock = Arc<tokio::sync::Mutex<()>>;

pub fn lock_for(source: SourceId) -> SourceLock {
    static LOCKS: OnceLock<Mutex<HashMap<&'static str, SourceLock>>> = OnceLock::new();
    let mut locks = LOCKS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    locks
        .entry(source.as_str())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}

#[derive(Debug, PartialEq, Eq)]
pub enum RefreshDecision {
    ReuseExisting,
    NeedRefresh,
}

pub fn decide_refresh(db_access: &str, current_token: &str) -> RefreshDecision {
    if db_access != current_token {
        RefreshDecision::ReuseExisting
    } else {
        RefreshDecision::NeedRefresh
    }
}

pub async fn refresh_locked(
    spec: &OAuthSpec,
    base: &str,
    pool: &SqlitePool,
    vault: &CryptoVault,
    stale_access: &str,
) -> Result<Zeroizing<String>, AppError> {
    let label = label_of(spec.id);
    let lock = lock_for(spec.id);
    let _guard = lock.lock().await;
    let stored = store::load_auth(pool, vault, spec.id)
        .await?
        .ok_or_else(|| AppError::Unauthorized(format!("No {label} authentication found")))?;
    if decide_refresh(&stored.access_token, stale_access) == RefreshDecision::ReuseExisting {
        return Ok(stored.access_token);
    }
    let (client_id, client_secret) = store::require_credentials(pool, vault, spec.id).await?;
    let mut tokens = refresh_token(
        spec,
        base,
        &client_id,
        &client_secret,
        &stored.refresh_token,
    )
    .await?;
    if tokens.refresh_token.is_empty() {
        tokens.refresh_token = stored.refresh_token.to_string();
    }
    store::save_auth(pool, vault, spec.id, &tokens)
        .await
        .map_err(|e| {
            AppError::Internal(format!(
                "save_tokens after refresh failed: {e}. Session OAuth corrompue, re-essaye la sync."
            ))
        })?;
    let noted = store::get_last_sync(pool, spec.id, REFRESH_EXPIRY).await?;
    if noted > 0 && noted <= chrono::Utc::now().timestamp() {
        store::set_last_sync(pool, spec.id, REFRESH_EXPIRY, 0).await?;
    }
    Ok(Zeroizing::new(std::mem::take(&mut tokens.access_token)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuse_when_db_token_differs() {
        assert_eq!(
            decide_refresh("new_token", "old_token"),
            RefreshDecision::ReuseExisting
        );
    }

    #[test]
    fn needs_refresh_when_db_token_matches_current() {
        assert_eq!(decide_refresh("same", "same"), RefreshDecision::NeedRefresh);
    }

    #[test]
    fn needs_refresh_when_both_empty() {
        assert_eq!(decide_refresh("", ""), RefreshDecision::NeedRefresh);
    }

    #[test]
    fn reuse_when_db_empty_current_nonempty() {
        assert_eq!(
            decide_refresh("", "old_token"),
            RefreshDecision::ReuseExisting
        );
    }

    #[test]
    fn needs_refresh_when_db_nonempty_current_empty() {
        assert_eq!(
            decide_refresh("db_token", "db_token"),
            RefreshDecision::NeedRefresh
        );
    }

    #[test]
    fn reuse_when_tokens_differ_by_whitespace() {
        assert_eq!(
            decide_refresh("tok ", "tok"),
            RefreshDecision::ReuseExisting
        );
    }

    #[test]
    fn a_source_has_one_lock_and_two_sources_have_two() {
        let a = lock_for(SourceId::WITHINGS);
        let b = lock_for(SourceId::WITHINGS);
        let c = lock_for(SourceId::new("oura"));
        assert!(Arc::ptr_eq(&a, &b));
        assert!(!Arc::ptr_eq(&a, &c));
    }
}
