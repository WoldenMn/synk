use crate::error::AppError;
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::{Connection, SqliteExecutor, SqlitePool};
use std::path::{Path, PathBuf};

const RETRY_ATTEMPTS: u32 = 5;
const RETRY_BASE_DELAY_MS: u64 = 50;

// Windows ne libere pas le handle d'un fichier de facon synchrone : apres la
// fermeture d'un pool SQLite, un renommage peut encore echouer un court instant
// avec ERROR_SHARING_VIOLATION (os error 32). Fermer le pool avant de restaurer
// a ete teste et ne suffit pas ; il faut attendre.
//
// Seule cette erreur est reessayee. Une permission refusee ne se resoudra pas
// d'elle-meme et doit remonter immediatement.
pub(crate) async fn retry_on_sharing_violation<T, F>(mut op: F) -> Result<T, std::io::Error>
where
    F: FnMut() -> Result<T, std::io::Error>,
{
    let mut delay = RETRY_BASE_DELAY_MS;
    let mut last_error = None;
    for attempt in 0..RETRY_ATTEMPTS {
        match op() {
            Ok(value) => return Ok(value),
            Err(e) if e.raw_os_error() == Some(32) => {
                last_error = Some(e);
                if attempt + 1 < RETRY_ATTEMPTS {
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    delay *= 2;
                }
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_error.unwrap_or_else(|| std::io::Error::from_raw_os_error(32)))
}

pub(crate) fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn vacuum_target<'a>(dest_path: &Path, partial: &'a Path) -> Result<&'a str, AppError> {
    let target = partial
        .to_str()
        .filter(|_| partial.is_absolute())
        .ok_or_else(|| {
            AppError::Validation(format!(
                "Sauvegarde impossible : le chemin {} doit être absolu et en UTF-8",
                dest_path.display()
            ))
        })?;
    let already_there = |p: &Path| {
        AppError::Internal(format!(
            "Sauvegarde impossible : {} existe déjà",
            p.display()
        ))
    };
    if dest_path.exists() {
        return Err(already_there(dest_path));
    }
    // Le .partial est cree vide et de facon exclusive : deux sauvegardes vers la
    // meme cible ne peuvent pas ecrire dans le meme fichier. VACUUM INTO accepte
    // une cible vide.
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(partial)
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => already_there(partial),
            _ => AppError::Internal(format!(
                "Sauvegarde impossible : {} ne peut pas être créé ({e})",
                partial.display()
            )),
        })?;
    Ok(target)
}

async fn vacuum_into<'c, E>(executor: E, target: &str) -> Result<(), AppError>
where
    E: SqliteExecutor<'c>,
{
    sqlx::query("VACUUM INTO ?")
        .bind(target)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(|e| AppError::Internal(format!("copie de la base échouée ({e})")))
}

async fn vacuum_into_with_own_connection(db_path: &Path, target: &str) -> Result<(), AppError> {
    let mut conn = SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(db_path))
        .await
        .map_err(|e| {
            AppError::Internal(format!(
                "ouverture de {} impossible ({e})",
                db_path.display()
            ))
        })?;
    let written = vacuum_into(&mut conn, target).await;
    let closed = conn.close().await.map_err(|e| {
        AppError::Internal(format!(
            "fermeture de {} impossible ({e})",
            db_path.display()
        ))
    });
    written.and(closed)
}

pub async fn verify_backup(path: &Path) -> Result<(), AppError> {
    let options = SqliteConnectOptions::new().filename(path).read_only(true);
    let mut conn = SqliteConnection::connect_with(&options)
        .await
        .map_err(|e| AppError::Internal(format!("ouverture de la copie impossible ({e})")))?;
    let report = sqlx::query_scalar::<_, String>("PRAGMA integrity_check")
        .fetch_all(&mut conn)
        .await;
    let tables =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM sqlite_master WHERE type = 'table'")
            .fetch_one(&mut conn)
            .await;
    let closed = conn.close().await;
    let report =
        report.map_err(|e| AppError::Internal(format!("contrôle d'intégrité impossible ({e})")))?;
    let tables =
        tables.map_err(|e| AppError::Internal(format!("lecture du schéma impossible ({e})")))?;
    closed.map_err(|e| AppError::Internal(format!("fermeture de la copie impossible ({e})")))?;
    if report != ["ok"] {
        return Err(AppError::Internal(format!(
            "contrôle d'intégrité en échec ({})",
            report.join(" ; ")
        )));
    }
    if tables == 0 {
        return Err(AppError::Internal(
            "la copie ne contient aucune table".into(),
        ));
    }
    Ok(())
}

async fn discard_partial(partial: &Path, reason: AppError) -> AppError {
    if !partial.exists() {
        return reason;
    }
    match retry_on_sharing_violation(|| std::fs::remove_file(partial)).await {
        Ok(()) => reason,
        Err(e) => AppError::Internal(format!(
            "{reason} ; le fichier temporaire {} n'a pas pu être supprimé ({e})",
            partial.display()
        )),
    }
}

// Une seule implementation de la copie de base, appelee par la sauvegarde
// manuelle ET par la sauvegarde pre-migration. Dupliquer cette logique a deja
// coute une sauvegarde automatique silencieusement amputee de ses dernieres
// ecritures : elle omettait le checkpoint que la sauvegarde manuelle, elle,
// effectuait.
pub async fn copy_database_safely(
    db_path: &Path,
    dest_path: &Path,
    pool: Option<&SqlitePool>,
) -> Result<(), AppError> {
    let partial = with_suffix(dest_path, ".partial");
    let target = vacuum_target(dest_path, &partial)?;
    let written = match pool {
        Some(p) => vacuum_into(p, target).await,
        None => vacuum_into_with_own_connection(db_path, target).await,
    };
    let verified = match written {
        Ok(()) => verify_backup(&partial).await,
        Err(e) => Err(e),
    };
    let finalized = match verified {
        Ok(()) => retry_on_sharing_violation(|| std::fs::rename(&partial, dest_path))
            .await
            .map_err(|e| AppError::Internal(format!("finalisation de la copie impossible ({e})"))),
        Err(reason) => Err(reason),
    };
    match finalized {
        Ok(()) => Ok(()),
        Err(reason) => Err(AppError::Internal(format!(
            "Sauvegarde vers {} impossible : {}",
            dest_path.display(),
            discard_partial(&partial, reason).await
        ))),
    }
}

pub async fn create_backup_in(
    app_data_dir: &Path,
    pool: Option<&SqlitePool>,
) -> Result<String, AppError> {
    let db_path = app_data_dir.join("synk.db");
    if !db_path.exists() {
        return Err(AppError::Internal("Base de données introuvable".into()));
    }

    let backups_dir = app_data_dir.join("backups");
    tokio::fs::create_dir_all(&backups_dir).await?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let backup_name = format!("synk_{}.db", timestamp);
    let backup_path = backups_dir.join(&backup_name);

    copy_database_safely(&db_path, &backup_path, pool).await?;

    super::backup_files::rotate_keep(&backups_dir, 5)?;

    Ok(backup_path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod retry_tests {
    use super::*;
    use std::cell::Cell;

    fn sharing_violation() -> std::io::Error {
        std::io::Error::from_raw_os_error(32)
    }

    #[tokio::test]
    async fn succeeds_after_a_few_sharing_violations() {
        let attempts = Cell::new(0);
        let result = retry_on_sharing_violation(|| {
            attempts.set(attempts.get() + 1);
            if attempts.get() < 4 {
                Err(sharing_violation())
            } else {
                Ok(())
            }
        })
        .await;
        assert!(result.is_ok(), "doit reussir apres trois echecs");
        assert_eq!(attempts.get(), 4);
    }

    #[tokio::test]
    async fn gives_up_after_the_last_attempt() {
        let attempts = Cell::new(0);
        let result: Result<(), std::io::Error> = retry_on_sharing_violation(|| {
            attempts.set(attempts.get() + 1);
            Err(sharing_violation())
        })
        .await;
        assert!(result.is_err(), "doit abandonner");
        assert_eq!(attempts.get(), 5, "cinq tentatives, pas une de plus");
    }

    #[tokio::test]
    async fn does_not_retry_another_error() {
        let attempts = Cell::new(0);
        let result: Result<(), std::io::Error> = retry_on_sharing_violation(|| {
            attempts.set(attempts.get() + 1);
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "acces refuse",
            ))
        })
        .await;
        assert!(result.is_err());
        assert_eq!(
            attempts.get(),
            1,
            "une erreur de permission ne doit pas etre reessayee"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn waits_fifty_then_doubles_and_never_sleeps_after_the_last_attempt() {
        let start = tokio::time::Instant::now();

        let result: Result<(), std::io::Error> =
            retry_on_sharing_violation(|| Err(sharing_violation())).await;

        assert!(result.is_err());
        assert_eq!(start.elapsed(), std::time::Duration::from_millis(750));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn waiting_for_a_shared_file_lets_other_tasks_run() {
        let ticks = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter = ticks.clone();
        let ticker = tokio::spawn(async move {
            for _ in 0..5 {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        });

        let result: Result<(), std::io::Error> =
            retry_on_sharing_violation(|| Err(sharing_violation())).await;

        assert!(result.is_err());
        assert_eq!(ticks.load(std::sync::atomic::Ordering::SeqCst), 5);
        ticker.await.unwrap();
    }
}
