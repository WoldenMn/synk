use serde::Serialize;
use sqlx::SqlitePool;

use super::CryptoVault;
use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InvalidatedSource {
    pub source: String,
    pub label: String,
    pub tokens: bool,
    pub credentials: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct InvalidatedSecrets {
    pub sources: Vec<InvalidatedSource>,
}

impl InvalidatedSecrets {
    pub fn any(&self) -> bool {
        !self.sources.is_empty()
    }

    fn entry(&mut self, source: &str) -> &mut InvalidatedSource {
        if let Some(i) = self.sources.iter().position(|s| s.source == source) {
            return &mut self.sources[i];
        }
        self.sources.push(InvalidatedSource {
            source: source.to_string(),
            label: label_of_str(source),
            tokens: false,
            credentials: false,
        });
        let last = self.sources.len() - 1;
        &mut self.sources[last]
    }

    fn mark_tokens(&mut self, source: &str) {
        self.entry(source).tokens = true;
    }

    fn mark_credentials(&mut self, source: &str) {
        self.entry(source).credentials = true;
    }
}

fn label_of_str(source: &str) -> String {
    crate::sources::find_source(source)
        .map(|s| s.label().to_string())
        .unwrap_or_else(|| source.to_string())
}

fn first_unreadable(vault: &CryptoVault, values: [&str; 2]) -> Option<AppError> {
    values
        .into_iter()
        .filter(|v| CryptoVault::is_encrypted(v))
        .find_map(|v| vault.open_str(v).err())
}

pub async fn invalidate_undecryptable_secrets(
    pool: &SqlitePool,
    vault: &CryptoVault,
) -> Result<InvalidatedSecrets, AppError> {
    let mut tx = pool.begin().await?;
    let mut report = InvalidatedSecrets::default();

    let rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT source_id, access_token, refresh_token FROM source_auth")
            .fetch_all(&mut *tx)
            .await?;
    for (source, access, refresh) in rows {
        if let Some(reason) = first_unreadable(vault, [&access, &refresh]) {
            sqlx::query("DELETE FROM source_auth WHERE source_id = ?1")
                .bind(&source)
                .execute(&mut *tx)
                .await?;
            log::warn!(
                "crypto: jetons {source} indechiffrables avec la cle courante ({reason}), \
                 supprimes : reconnexion requise"
            );
            report.mark_tokens(&source);
        }
    }

    let rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT source_id, client_id, client_secret FROM source_credentials")
            .fetch_all(&mut *tx)
            .await?;
    for (source, client_id, client_secret) in rows {
        if let Some(reason) = first_unreadable(vault, [&client_id, &client_secret]) {
            sqlx::query(
                "UPDATE source_credentials \
                 SET client_id = '', client_secret = '', updated_at = CURRENT_TIMESTAMP \
                 WHERE source_id = ?1",
            )
            .bind(&source)
            .execute(&mut *tx)
            .await?;
            log::warn!(
                "crypto: credentials {source} indechiffrables avec la cle courante ({reason}), \
                 effaces : ressaisie du Client ID et du Client Secret requise"
            );
            report.mark_credentials(&source);
        }
    }

    tx.commit().await?;
    Ok(report)
}
