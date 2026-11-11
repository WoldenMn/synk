use crate::error::AppError;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const KIND_DIRS: [(&str, &str); 3] = [("auto", "auto"), ("quotidienne", "daily"), ("manuelle", "")];

pub fn db_files_newest_first(dir: &Path) -> Result<Vec<(PathBuf, SystemTime)>, AppError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let read_error = |e: std::io::Error| {
        AppError::Internal(format!("lecture de {} impossible ({e})", dir.display()))
    };
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(read_error)? {
        let entry = entry.map_err(read_error)?;
        let path = entry.path();
        if path.extension().is_some_and(|x| x == "db") {
            let modified = entry.metadata().and_then(|m| m.modified()).map_err(|e| {
                AppError::Internal(format!("date de {} illisible ({e})", path.display()))
            })?;
            files.push((path, modified));
        }
    }
    files.sort_by_key(|f| std::cmp::Reverse(f.1));
    Ok(files)
}

pub fn rotate_keep(dir: &Path, keep: usize) -> Result<Vec<PathBuf>, AppError> {
    let mut removed = Vec::new();
    for (path, _) in db_files_newest_first(dir)?.into_iter().skip(keep) {
        match std::fs::remove_file(&path) {
            Ok(()) => removed.push(path),
            Err(e) => log::warn!("[backup] rotation : {} non supprime ({e})", path.display()),
        }
    }
    Ok(removed)
}

pub fn all_backups_newest_first(
    app_data_dir: &Path,
) -> Result<Vec<(PathBuf, SystemTime, &'static str)>, AppError> {
    let root = app_data_dir.join("backups");
    let mut all = Vec::new();
    for (kind, sub) in KIND_DIRS {
        let dir = if sub.is_empty() {
            root.clone()
        } else {
            root.join(sub)
        };
        for (path, modified) in db_files_newest_first(&dir)? {
            all.push((path, modified, kind));
        }
    }
    all.sort_by_key(|f| std::cmp::Reverse(f.1));
    Ok(all)
}

#[derive(Debug, serde::Serialize, PartialEq, Eq)]
pub struct BackupInfo {
    pub path: String,
    pub kind: &'static str,
    pub size_bytes: u64,
    pub modified: String,
}

pub fn list_backups_inner(app_data_dir: &Path) -> Result<Vec<BackupInfo>, AppError> {
    all_backups_newest_first(app_data_dir)?
        .into_iter()
        .map(|(path, modified, kind)| {
            let size_bytes = std::fs::metadata(&path).map(|m| m.len()).map_err(|e| {
                AppError::Internal(format!("taille de {} illisible ({e})", path.display()))
            })?;
            Ok(BackupInfo {
                path: path.to_string_lossy().into_owned(),
                kind,
                size_bytes,
                modified: chrono::DateTime::<chrono::Utc>::from(modified)
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            })
        })
        .collect()
}
