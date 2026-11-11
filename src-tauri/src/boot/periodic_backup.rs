use crate::error::AppError;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

pub const PERIODIC_KEEP: usize = 14;
const PERIODIC_MIN_AGE: Duration = Duration::from_secs(20 * 3600);

#[derive(Default)]
pub struct BackupFailure(Mutex<Option<String>>);

impl BackupFailure {
    pub fn record(&self, message: &str) -> bool {
        let mut slot = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let is_new = slot.as_deref() != Some(message);
        *slot = Some(message.to_string());
        is_new
    }

    pub fn clear(&self) {
        *self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }

    pub fn current(&self) -> Option<AppError> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
            .map(|message| AppError::Internal(message).redacted())
    }
}

#[tauri::command]
pub fn get_backup_failure(failure: tauri::State<'_, BackupFailure>) -> Option<AppError> {
    failure.current()
}

pub fn backup_failure_payload(message: &str) -> serde_json::Value {
    AppError::Internal(message.to_string()).into_event_payload()
}

#[derive(Debug, PartialEq, Eq)]
pub enum PeriodicOutcome {
    Written(PathBuf),
    Fresh,
}

pub async fn periodic_backup_inner(
    pool: &SqlitePool,
    app_data_dir: &Path,
    now: SystemTime,
) -> Result<PeriodicOutcome, AppError> {
    let dir = app_data_dir.join("backups").join("daily");
    tokio::fs::create_dir_all(&dir).await.map_err(|e| {
        AppError::Internal(format!(
            "dossier {} impossible a creer ({e})",
            dir.display()
        ))
    })?;
    let existing = crate::services::backup_files::db_files_newest_first(&dir)?;
    if let Some((_, modified)) = existing.first() {
        if now.duration_since(*modified).unwrap_or_default() < PERIODIC_MIN_AGE {
            return Ok(PeriodicOutcome::Fresh);
        }
    }
    let stamp = chrono::DateTime::<chrono::Local>::from(now).format("%Y-%m-%d_%H-%M-%S");
    let dest = dir.join(format!("synk_{stamp}_quotidienne.db"));
    crate::services::backup::copy_database_safely(&app_data_dir.join("synk.db"), &dest, Some(pool))
        .await?;
    crate::services::backup_files::rotate_keep(&dir, PERIODIC_KEEP)?;
    Ok(PeriodicOutcome::Written(dest))
}
