use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use sqlx::SqlitePool;

use crate::error::AppError;
use crate::sources::import::pipeline::{ImportProgress, ImportReport};
use crate::sources::import::table::{import_csv, CsvConfig};
use crate::sources::{source_by_name, ImportContext, SourceKind};

#[derive(Default)]
pub struct ImportControl {
    running: Mutex<Option<Arc<AtomicBool>>>,
}

impl ImportControl {
    fn begin(&self) -> Result<Arc<AtomicBool>, AppError> {
        let mut slot = self
            .running
            .lock()
            .map_err(|_| AppError::Internal("etat de l'import indisponible".into()))?;
        if slot.is_some() {
            return Err(AppError::Validation("un import est déjà en cours".into()));
        }
        let cancel = Arc::new(AtomicBool::new(false));
        *slot = Some(Arc::clone(&cancel));
        Ok(cancel)
    }

    fn end(&self) {
        if let Ok(mut slot) = self.running.lock() {
            *slot = None;
        }
    }

    pub fn cancel(&self) -> bool {
        let Ok(slot) = self.running.lock() else {
            return false;
        };
        match slot.as_ref() {
            Some(flag) => {
                flag.store(true, Ordering::Relaxed);
                true
            }
            None => false,
        }
    }
}

pub async fn import_file(
    control: &ImportControl,
    pool: &SqlitePool,
    source: &str,
    path: &str,
    on_progress: &(dyn Fn(ImportProgress) + Sync),
) -> Result<ImportReport, AppError> {
    let found = source_by_name(source)?;
    if !matches!(found.kind(), SourceKind::File(_)) {
        return Err(AppError::Validation(format!(
            "la source {source} ne s'importe pas depuis un fichier"
        )));
    }
    let cancel = control.begin()?;
    let report = match found.import(ImportContext {
        pool,
        path,
        cancel,
        on_progress,
    }) {
        Some(future) => future.await,
        None => Err(AppError::Internal(format!(
            "aucun lecteur de fichier pour la source {source}"
        ))),
    };
    control.end();
    report
}

pub async fn import_csv_file(
    control: &ImportControl,
    pool: &SqlitePool,
    path: &str,
    config: CsvConfig,
    on_progress: &(dyn Fn(ImportProgress) + Sync),
) -> Result<ImportReport, AppError> {
    let cancel = control.begin()?;
    let report = import_csv(pool, path, config, cancel, on_progress).await;
    control.end();
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_import_runs_at_a_time_and_cancel_reaches_it() {
        let control = ImportControl::default();
        assert!(!control.cancel());
        let flag = control.begin().expect("premier import");
        assert!(control.begin().is_err());
        assert!(!flag.load(Ordering::Relaxed));
        assert!(control.cancel());
        assert!(flag.load(Ordering::Relaxed));
        control.end();
        assert!(!control.cancel());
        assert!(control.begin().is_ok());
    }
}
