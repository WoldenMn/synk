use crate::services::{audit, error_log, sync_log};
use sqlx::SqlitePool;

pub const SYNC_LOG_KEEP_ROWS: i64 = 1_000;
pub const SYNC_LOG_KEEP_DAYS: i64 = 90;
pub const ERROR_LOG_KEEP_ROWS: i64 = 2_000;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RetentionReport {
    pub sync_log_removed: u64,
    pub audit_removed: Option<i64>,
}

pub async fn run(pool: &SqlitePool, now_iso: &str) -> RetentionReport {
    let mut report = RetentionReport::default();
    match sync_log::purge(pool, SYNC_LOG_KEEP_ROWS, SYNC_LOG_KEEP_DAYS).await {
        Ok(removed) => {
            if removed > 0 {
                log::info!("[boot] sync_log: {removed} lignes retirees");
            }
            report.sync_log_removed = removed;
        }
        Err(e) => log::warn!("[boot] sync_log purge failed: {e}"),
    }
    report.audit_removed = purge_audit_if_due(pool, now_iso).await;
    if let Err(e) = error_log::auto_purge(pool, ERROR_LOG_KEEP_ROWS).await {
        log::warn!("auto_purge_error_log failed: {e}");
    }
    report
}

async fn purge_audit_if_due(pool: &SqlitePool, now_iso: &str) -> Option<i64> {
    let config = match audit::get_audit_purge_config(pool).await {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[boot] auto-purge config read failed: {e}");
            return None;
        }
    };
    if !config.enabled {
        log::info!("[boot] auto-purge disabled, skipping");
        return None;
    }
    let metrics = match audit::get_audit_metrics(pool).await {
        Ok(m) => m,
        Err(e) => {
            log::warn!("[boot] auto-purge metrics read failed: {e}");
            return None;
        }
    };
    let Some(days) = audit::should_auto_purge(&metrics, &config, now_iso) else {
        log::info!("[boot] auto-purge: no action needed");
        return None;
    };
    match audit::purge_audit_log(pool, days).await {
        Ok(deleted) => {
            log::info!("[boot] auto-purge: {deleted} entrees supprimees (> {days} jours)");
            Some(deleted)
        }
        Err(e) => {
            log::warn!("[boot] auto-purge failed: {e}");
            None
        }
    }
}
