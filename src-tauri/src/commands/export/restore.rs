use crate::error::AppError;
use crate::services::backup::{retry_on_sharing_violation, verify_backup, with_suffix};
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Debug, serde::Serialize)]
pub struct RestoreReport {
    pub restored_from: String,
    pub broken_kept_at: String,
    pub skipped: Vec<String>,
}

type Moves = Vec<(PathBuf, PathBuf)>;

async fn put_back(moves: &[(PathBuf, PathBuf)]) -> String {
    let mut leftovers = String::new();
    for (original, moved) in moves.iter().rev() {
        if let Err(undo) = retry_on_sharing_violation(|| std::fs::rename(moved, original)).await {
            leftovers.push_str(&format!(
                " ; {} est resté en {} ({undo})",
                original.display(),
                moved.display()
            ));
        }
    }
    leftovers
}

async fn move_aside(db_path: &Path, broken_path: &Path) -> Result<Moves, AppError> {
    move_aside_with(db_path, broken_path, |from, to| async move {
        retry_on_sharing_violation(|| std::fs::rename(&from, &to)).await
    })
    .await
}

async fn move_aside_with<F, Fut>(
    db_path: &Path,
    broken_path: &Path,
    rename: F,
) -> Result<Moves, AppError>
where
    F: Fn(PathBuf, PathBuf) -> Fut,
    Fut: std::future::Future<Output = std::io::Result<()>>,
{
    let mut moves: Moves = Vec::new();
    for suffix in ["", "-wal", "-shm"] {
        let current = with_suffix(db_path, suffix);
        let aside = with_suffix(broken_path, suffix);
        match rename(current.clone(), aside.clone()).await {
            Ok(()) => moves.push((current, aside)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::info!("[restore] {} absent, rien a preserver", current.display());
            }
            Err(e) => {
                return Err(AppError::Internal(format!(
                    "Impossible de préserver la base courante ({}) : {e}{}",
                    current.display(),
                    put_back(&moves).await
                )));
            }
        }
    }
    Ok(moves)
}

#[cfg(test)]
mod move_aside_tests {
    use super::*;

    #[tokio::test]
    async fn a_journal_that_vanishes_before_the_move_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("synk.db");
        let broken = dir.path().join("synk.db.broken-x");
        for suffix in ["", "-wal", "-shm"] {
            std::fs::write(with_suffix(&db, suffix), b"x").unwrap();
        }
        let wal = with_suffix(&db, "-wal");

        let moves = move_aside_with(&db, &broken, |from, to| {
            let wal = wal.clone();
            async move {
                if from == wal {
                    std::fs::remove_file(&from).unwrap();
                }
                std::fs::rename(from, to)
            }
        })
        .await
        .unwrap();

        assert_eq!(moves.len(), 2);
        assert!(broken.exists());
        assert!(with_suffix(&broken, "-shm").exists());
        assert!(!with_suffix(&broken, "-wal").exists());
    }
}

fn file_label(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

// Copie la sauvegarde a cote de la base, sous un nom temporaire, et verifie
// cette copie : c'est elle qui deviendra synk.db, pas le fichier d'origine.
async fn stage_copy(backup: &Path, staged: &Path) -> Result<(), AppError> {
    if tokio::fs::try_exists(staged).await.unwrap_or(false) {
        retry_on_sharing_violation(|| std::fs::remove_file(staged))
            .await
            .map_err(|e| {
                AppError::Internal(format!(
                    "copie temporaire précédente impossible à retirer ({}) : {e}",
                    staged.display()
                ))
            })?;
    }
    tokio::fs::copy(backup, staged)
        .await
        .map_err(|e| AppError::Internal(format!("copie impossible ({e})")))?;
    if let Err(reason) = verify_backup(staged).await {
        let cleanup = retry_on_sharing_violation(|| std::fs::remove_file(staged))
            .await
            .err()
            .map(|e| format!(" ; {} n'a pas pu être supprimé ({e})", staged.display()))
            .unwrap_or_default();
        return Err(AppError::Internal(format!("{reason}{cleanup}")));
    }
    Ok(())
}

// Fn testable sans AppHandle : logique centrale de restore_latest_backup.
// Restaure la sauvegarde la plus recente qui passe le controle d'integrite ;
// une sauvegarde illisible est ecartee et nommee dans le rapport. La base
// courante n'est deplacee qu'une fois la copie de remplacement prete et
// verifiee, et remise en place si la pose echoue.
pub async fn restore_latest_backup_inner(app_data_dir: &Path) -> Result<RestoreReport, AppError> {
    let db_path = app_data_dir.join("synk.db");
    let staged = app_data_dir.join("synk.db.restoring");
    let candidates: Vec<PathBuf> =
        crate::services::backup_files::all_backups_newest_first(app_data_dir)?
            .into_iter()
            .map(|(path, _, _)| path)
            .collect();
    if candidates.is_empty() {
        return Err(AppError::Internal("Aucune sauvegarde disponible".into()));
    }

    let mut skipped = Vec::new();
    let mut chosen = None;
    for candidate in candidates {
        match stage_copy(&candidate, &staged).await {
            Ok(()) => {
                chosen = Some(candidate);
                break;
            }
            Err(reason) => {
                log::warn!(
                    "[restore] sauvegarde ecartee {} : {reason}",
                    candidate.display()
                );
                skipped.push(format!("{} ({reason})", file_label(&candidate)));
            }
        }
    }
    let restored_from = chosen.ok_or_else(|| {
        AppError::Internal(format!(
            "Aucune sauvegarde utilisable, la base actuelle n'a pas été touchée : {}",
            skipped.join(" ; ")
        ))
    })?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let broken_path = app_data_dir.join(format!("synk.db.broken-{timestamp}"));
    let moves = move_aside(&db_path, &broken_path).await?;

    if let Err(e) = retry_on_sharing_violation(|| std::fs::rename(&staged, &db_path)).await {
        let leftovers = put_back(&moves).await;
        let state = if leftovers.is_empty() {
            ", la base actuelle a été remise en place".to_string()
        } else {
            leftovers
        };
        return Err(AppError::Internal(format!(
            "Restauration échouée ({e}){state}"
        )));
    }

    Ok(RestoreReport {
        restored_from: restored_from.to_string_lossy().into_owned(),
        broken_kept_at: broken_path.to_string_lossy().into_owned(),
        skipped,
    })
}

#[tauri::command]
pub async fn restore_latest_backup(app: tauri::AppHandle) -> Result<RestoreReport, AppError> {
    let app_data_dir = app.path().app_data_dir()?;
    restore_latest_backup_inner(&app_data_dir).await
}

#[cfg(test)]
mod runtime_availability_tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    #[tokio::test(flavor = "current_thread")]
    async fn the_runtime_keeps_ticking_while_a_large_backup_is_copied() {
        let dir = tempfile::tempdir().unwrap();
        let backup = dir.path().join("big.db");
        let staged = dir.path().join("synk.db.restoring");
        let chunk = vec![7u8; 1 << 20];
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&backup).unwrap();
            for _ in 0..192 {
                f.write_all(&chunk).unwrap();
            }
        }
        let reference = dir.path().join("reference.db");
        let started = Instant::now();
        std::fs::copy(&backup, &reference).unwrap();
        let copy_time = started.elapsed();
        assert!(
            copy_time > Duration::from_millis(40),
            "copie de reference trop rapide pour mesurer ({copy_time:?})"
        );

        let done = Arc::new(AtomicBool::new(false));
        let flag = done.clone();
        let ticker = tokio::spawn(async move {
            let mut worst = Duration::ZERO;
            let mut last = Instant::now();
            while !flag.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(1)).await;
                worst = worst.max(last.elapsed());
                last = Instant::now();
            }
            worst
        });
        tokio::task::yield_now().await;
        assert!(stage_copy(&backup, &staged).await.is_err());
        done.store(true, Ordering::Relaxed);
        let worst = ticker.await.unwrap();

        assert!(
            worst < copy_time * 3 / 5,
            "le runtime est reste bloque {worst:?} pour une copie de {copy_time:?}"
        );
    }
}
