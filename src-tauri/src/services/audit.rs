use crate::error::AppError;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditMetrics {
    pub total_count: i64,
    pub insert_count: i64,
    pub delete_count: i64,
    pub oldest_at: Option<String>,
    pub newest_at: Option<String>,
}

/// Variante pour les appelants dont l'action principale a deja reussi : un
/// journal d'audit indisponible ne doit pas faire echouer une suppression, mais
/// son echec reste trace.
pub async fn record_audit_or_warn(
    pool: &SqlitePool,
    action: &str,
    entity: &str,
    entity_id: Option<i64>,
    details: Option<&str>,
) {
    if let Err(e) = record_audit(pool, action, entity, entity_id, details).await {
        log::warn!("[audit] entree {action} {entity} non enregistree : {e}");
    }
}

/// Inserer une entree dans audit_log. Helper interne, appelable depuis les autres commandes.
pub async fn record_audit(
    pool: &SqlitePool,
    action: &str,
    entity: &str,
    entity_id: Option<i64>,
    details: Option<&str>,
) -> Result<i64, AppError> {
    let result = sqlx::query(
        "INSERT INTO audit_log (action, entity, entity_id, details) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(action)
    .bind(entity)
    .bind(entity_id)
    .bind(details)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_audit_metrics(pool: &SqlitePool) -> Result<AuditMetrics, AppError> {
    let metrics = sqlx::query_as::<_, AuditMetrics>(
        "SELECT
            COUNT(*) AS total_count,
            COALESCE(SUM(CASE WHEN action = 'insert' THEN 1 ELSE 0 END), 0) AS insert_count,
            COALESCE(SUM(CASE WHEN action = 'delete' THEN 1 ELSE 0 END), 0) AS delete_count,
            MIN(created_at) AS oldest_at,
            MAX(created_at) AS newest_at
         FROM audit_log",
    )
    .fetch_one(pool)
    .await?;
    Ok(metrics)
}

pub async fn purge_audit_log(pool: &SqlitePool, older_than_days: i64) -> Result<i64, AppError> {
    if older_than_days < 0 {
        return Err(AppError::Validation(
            "older_than_days doit etre positif".into(),
        ));
    }
    if older_than_days > 36500 {
        return Err(AppError::Validation("Retention max 36500 jours".into()));
    }
    let cutoff = format!("-{} days", older_than_days);
    let result = sqlx::query("DELETE FROM audit_log WHERE created_at < datetime('now', ?1)")
        .bind(&cutoff)
        .execute(pool)
        .await?;
    let deleted = result.rows_affected() as i64;
    record_audit_or_warn(
        pool,
        "purge",
        "audit_log",
        None,
        Some(&format!(
            "{} entrees supprimees (> {} jours)",
            deleted, older_than_days
        )),
    )
    .await;
    Ok(deleted)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AuditPurgeConfig {
    pub retention_days: i64,
    pub max_entries: i64,
    pub enabled: bool,
}

impl Default for AuditPurgeConfig {
    fn default() -> Self {
        Self {
            retention_days: 180,
            max_entries: 10000,
            enabled: true,
        }
    }
}

fn compute_cutoff(now_iso: &str, days: i64) -> String {
    let now = chrono::NaiveDateTime::parse_from_str(now_iso, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| chrono::Utc::now().naive_utc());
    let cutoff = now - chrono::Duration::days(days);
    cutoff.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn should_auto_purge(
    metrics: &AuditMetrics,
    config: &AuditPurgeConfig,
    now_iso: &str,
) -> Option<i64> {
    if !config.enabled {
        return None;
    }
    if metrics.total_count > config.max_entries {
        return Some(config.retention_days);
    }
    if let Some(oldest) = &metrics.oldest_at {
        let cutoff = compute_cutoff(now_iso, config.retention_days);
        if oldest.as_str() < cutoff.as_str() {
            return Some(config.retention_days);
        }
    }
    None
}

pub async fn get_audit_purge_config(pool: &SqlitePool) -> Result<AuditPurgeConfig, AppError> {
    let row: (i64, i64, i64) = sqlx::query_as(
        "SELECT audit_retention_days, audit_max_entries, audit_auto_purge_enabled
         FROM settings WHERE id = 1",
    )
    .fetch_one(pool)
    .await?;
    Ok(AuditPurgeConfig {
        retention_days: row.0,
        max_entries: row.1,
        enabled: row.2 != 0,
    })
}

pub async fn set_audit_purge_config(
    pool: &SqlitePool,
    config: AuditPurgeConfig,
) -> Result<(), AppError> {
    if config.retention_days < 0 {
        return Err(AppError::Validation(
            "retention_days doit etre positif".into(),
        ));
    }
    if config.max_entries < 0 {
        return Err(AppError::Validation("max_entries doit etre positif".into()));
    }
    let result = sqlx::query(
        "UPDATE settings SET
            audit_retention_days = ?1,
            audit_max_entries = ?2,
            audit_auto_purge_enabled = ?3
         WHERE id = 1",
    )
    .bind(config.retention_days)
    .bind(config.max_entries)
    .bind(if config.enabled { 1i64 } else { 0i64 })
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            entity: "settings",
            id: 1,
        });
    }
    Ok(())
}
