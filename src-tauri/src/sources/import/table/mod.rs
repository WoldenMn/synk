pub mod config;
pub mod convert;
pub mod dates;
pub mod inspect;
pub mod read;
pub mod source;
pub mod store;
pub mod targets;

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use sqlx::SqlitePool;

use super::pipeline::{
    checked_path, file_size, import_file, send_all, ImportProgress, ImportReport, Produced, Sender,
    Shared,
};
use crate::error::AppError;
use crate::sources::SourceId;

pub use config::{signature, ColumnMap, CsvConfig};
pub use inspect::Inspection;
pub use source::CSV;

const ALLOWED: &[&str] = &["csv", "txt"];

fn load(path: &Path) -> Result<String, AppError> {
    let size = std::fs::metadata(path)
        .map_err(|e| AppError::Validation(format!("fichier illisible : {e}")))?
        .len();
    if size > read::MAX_BYTES {
        return Err(AppError::Validation(
            "fichier CSV trop volumineux (plus de 256 Mo)".into(),
        ));
    }
    let bytes = std::fs::read(path)
        .map_err(|e| AppError::Validation(format!("fichier illisible : {e}")))?;
    if bytes.iter().take(8192).any(|b| *b == 0) {
        return Err(AppError::Validation(
            "ce fichier n'est pas un CSV : il contient des données binaires".into(),
        ));
    }
    Ok(read::decode_text(&bytes))
}

pub fn inspect_file(path: &str) -> Result<Inspection, AppError> {
    let path = checked_path(path, ALLOWED, ".csv et .txt")?;
    inspect::inspect_text(&load(&path)?)
}

fn produce(
    path: &Path,
    config: &CsvConfig,
    tx: &Sender,
    cancel: &AtomicBool,
    shared: &Shared,
) -> Result<Produced, AppError> {
    file_size(path, shared)?;
    let converted = convert::convert(&load(path)?, config)?;
    if cancel.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(Produced {
            cancelled: true,
            skipped: Default::default(),
        });
    }
    send_all(tx, converted.outs)?;
    Ok(Produced {
        cancelled: false,
        skipped: converted.skipped,
    })
}

pub async fn import_csv(
    pool: &SqlitePool,
    path: &str,
    config: CsvConfig,
    cancel: Arc<AtomicBool>,
    on_progress: &(dyn Fn(ImportProgress) + Sync),
) -> Result<ImportReport, AppError> {
    let path = checked_path(path, ALLOWED, ".csv et .txt")?;
    import_file(
        pool,
        SourceId::CSV,
        cancel,
        on_progress,
        move |tx, cancel, shared| produce(&path, &config, tx, cancel, shared),
    )
    .await
}
