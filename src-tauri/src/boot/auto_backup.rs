use super::{BootError, EXPECTED_LATEST_VERSION};
use std::path::{Path, PathBuf};

pub const BACKUP_ROLLING_KEEP: usize = 3;

/// Lit la schema_version courante. Retourne 0 si le fichier ou la table n'existe pas
/// (base vierge ou pre-versionnee) ; une base illisible est une erreur, jamais la version 0.
pub async fn read_current_schema_version(db_path: &Path) -> Result<i64, BootError> {
    if !db_path.exists() {
        return Ok(0);
    }
    let unreadable = |e: sqlx::Error| BootError::DbLocked {
        message: format!("lecture de la version du schema impossible : {e}"),
    };
    let url = format!("sqlite:{}?mode=ro", db_path.display());
    let pool = sqlx::SqlitePool::connect(&url).await.map_err(unreadable)?;
    let version: Result<i64, sqlx::Error> = async {
        let has_table: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'schema_version'",
        )
        .fetch_one(&pool)
        .await?;
        if has_table == 0 {
            return Ok(0);
        }
        let v: Option<i64> = sqlx::query_scalar("SELECT MAX(version) FROM schema_version")
            .fetch_one(&pool)
            .await?;
        Ok(v.unwrap_or(0))
    }
    .await;
    pool.close().await;
    version.map_err(unreadable)
}

/// Cree un backup `synk_{ISO}_pre-v{n}.db` dans `app_data_dir/backups/auto/`.
/// No-op si DB vierge (path inexistant) OU si schema_version >= EXPECTED_LATEST_VERSION.
/// Rolling : garde les BACKUP_ROLLING_KEEP plus recents (par mtime), supprime le reste.
pub async fn backup_before_migration(
    db_path: &Path,
    app_data_dir: &Path,
) -> Result<Option<PathBuf>, BootError> {
    if !db_path.exists() {
        return Ok(None);
    }
    let current_version = read_current_schema_version(db_path).await?;
    if current_version >= EXPECTED_LATEST_VERSION {
        return Ok(None);
    }

    let backups_dir = app_data_dir.join("backups").join("auto");
    tokio::fs::create_dir_all(&backups_dir)
        .await
        .map_err(|e| BootError::AppDataDir {
            message: format!("Cannot create backups dir: {e}"),
        })?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let backup_name = format!("synk_{timestamp}_pre-v{current_version}.db");
    let backup_path = backups_dir.join(&backup_name);

    crate::services::backup::copy_database_safely(db_path, &backup_path, None)
        .await
        .map_err(|e| BootError::BackupFailed {
            message: e.to_string(),
        })?;

    crate::services::backup_files::rotate_keep(&backups_dir, BACKUP_ROLLING_KEEP).map_err(|e| {
        BootError::AppDataDir {
            message: e.to_string(),
        }
    })?;

    Ok(Some(backup_path))
}
