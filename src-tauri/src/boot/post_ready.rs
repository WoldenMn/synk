use crate::error::AppError;
use sqlx::SqlitePool;

pub fn spawn_post_ready_seed_then_backup(handle: tauri::AppHandle, pool: SqlitePool) {
    tauri::async_runtime::spawn(async move {
        crate::db::run_post_ready_seed(&pool).await;
        super::run_every(super::BACKUP_RECHECK_INTERVAL, || {
            run_periodic_backup(&handle, &pool)
        })
        .await;
    });
}

async fn run_periodic_backup(handle: &tauri::AppHandle, pool: &SqlitePool) {
    use tauri::Manager;
    let outcome = match handle.path().app_data_dir() {
        Ok(dir) => super::periodic_backup_inner(pool, &dir, std::time::SystemTime::now()).await,
        Err(e) => Err(AppError::Internal(format!(
            "dossier de donnees introuvable ({e})"
        ))),
    };
    match outcome {
        Ok(super::PeriodicOutcome::Written(path)) => {
            log::info!(
                "[backup] sauvegarde quotidienne ecrite : {}",
                path.display()
            );
            handle.state::<super::BackupFailure>().clear();
            if let Err(e) = crate::db::optimize(pool).await {
                log::warn!("[db] PRAGMA optimize quotidien en echec : {e}");
            }
            let now_iso = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            crate::services::retention::run(pool, &now_iso).await;
        }
        Ok(super::PeriodicOutcome::Fresh) => {
            handle.state::<super::BackupFailure>().clear();
            log::info!("[backup] sauvegarde quotidienne a jour");
        }
        Err(error) => report_backup_failure(handle, pool, &error.detail()).await,
    }
}

async fn report_backup_failure(handle: &tauri::AppHandle, pool: &SqlitePool, message: &str) {
    use tauri::{Emitter, Manager};
    log::error!("[backup] sauvegarde quotidienne impossible : {message}");
    if !handle.state::<super::BackupFailure>().record(message) {
        return;
    }
    if let Err(e) = crate::services::error_log::log_error(
        pool,
        "backup",
        "BACKUP_FAILED",
        message,
        Some("sauvegarde quotidienne"),
    )
    .await
    {
        log::warn!("[backup] echec non consigne dans error_log : {e}");
    }
    if let Err(e) = handle.emit("backup-failed", super::backup_failure_payload(message)) {
        log::warn!("[backup] evenement backup-failed non emis : {e}");
    }
}

/// Tache detachee post-ready : retention des journaux. Best-effort : un echec est
/// journalise et ne bloque jamais le boot.
pub fn spawn_post_ready_auto_purge(pool: SqlitePool) {
    tauri::async_runtime::spawn(async move {
        let now_iso = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        crate::services::retention::run(&pool, &now_iso).await;
    });
}
