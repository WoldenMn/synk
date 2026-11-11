pub mod map;
pub mod read;
pub mod source;

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use sqlx::SqlitePool;

use super::pipeline::{
    checked_path, file_size, import_file, send_all, ImportProgress, ImportReport, Produced, Sender,
    Shared,
};
use super::rows::Skipped;
use crate::error::AppError;
use crate::sources::SourceId;

pub use source::FIT;

fn produce(
    path: &Path,
    tx: &Sender,
    cancel: &AtomicBool,
    shared: &Shared,
) -> Result<Produced, AppError> {
    file_size(path, shared)?;
    let size = std::fs::metadata(path)
        .map_err(|e| AppError::Validation(format!("fichier illisible : {e}")))?
        .len();
    read::check_size(size)?;
    let bytes = std::fs::read(path)
        .map_err(|e| AppError::Validation(format!("fichier illisible : {e}")))?;
    let records = read::decode(&bytes)?;
    let mut mapper = map::Mapper::default();
    for record in &records {
        mapper.feed(record);
    }
    let (outs, skipped) = mapper.finish();
    if cancel.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(Produced {
            cancelled: true,
            skipped: Skipped::default(),
        });
    }
    if outs.is_empty() && skipped.total() == 0 {
        let mut skipped = Skipped::default();
        skipped.add("fichier FIT : aucune séance, pesée, tension ni nuit reconnue");
        return Ok(Produced {
            cancelled: false,
            skipped,
        });
    }
    send_all(tx, outs)?;
    Ok(Produced {
        cancelled: false,
        skipped,
    })
}

pub async fn import_fit(
    pool: &SqlitePool,
    path: &str,
    cancel: Arc<AtomicBool>,
    on_progress: &(dyn Fn(ImportProgress) + Sync),
) -> Result<ImportReport, AppError> {
    let path = checked_path(path, &["fit"], ".fit")?;
    import_file(
        pool,
        SourceId::FIT,
        cancel,
        on_progress,
        move |tx, cancel, shared| produce(&path, tx, cancel, shared),
    )
    .await
}
