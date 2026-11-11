use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sqlx::SqlitePool;

use super::mapping::{wants, Mapper};
use super::open::with_export_limited;
use super::reader::{parse_export_limited, ParseEnd};
use crate::error::AppError;
use crate::sources::import::limits::Limits;
use crate::sources::import::pipeline::{checked_path, import_file, Produced, Sender, Shared};
use crate::sources::import::rows::Out;
use crate::sources::SourceId;

pub use crate::sources::import::pipeline::{ImportProgress, ImportReport, BATCH};

fn produce(
    path: &Path,
    limits: &Limits,
    tx: &Sender,
    cancel: &AtomicBool,
    shared: &Shared,
) -> Result<Produced, AppError> {
    with_export_limited(path, Arc::clone(&shared.seen), limits, |source, size| {
        shared.size.store(size, Ordering::Relaxed);
        let mut mapper = Mapper::default();
        let mut buffer: Vec<Out> = Vec::with_capacity(BATCH);
        let closed = || AppError::Internal("import interrompu".into());
        let end = parse_export_limited(source, limits, wants, cancel, |item| {
            mapper.feed(item, &mut buffer);
            if buffer.len() >= BATCH {
                tx.blocking_send(std::mem::take(&mut buffer))
                    .map_err(|_| closed())?;
            }
            Ok(())
        })?;
        let skipped = mapper.finish(&mut buffer);
        if end == ParseEnd::Done && !buffer.is_empty() {
            tx.blocking_send(buffer).map_err(|_| closed())?;
        }
        Ok(Produced {
            cancelled: end == ParseEnd::Cancelled,
            skipped,
        })
    })
}

pub async fn import_apple_health(
    pool: &SqlitePool,
    path: &str,
    cancel: Arc<AtomicBool>,
    on_progress: &(dyn Fn(ImportProgress) + Sync),
) -> Result<ImportReport, AppError> {
    import_apple_health_with(pool, path, Limits::APPLE_HEALTH, cancel, on_progress).await
}

pub async fn import_apple_health_with(
    pool: &SqlitePool,
    path: &str,
    limits: Limits,
    cancel: Arc<AtomicBool>,
    on_progress: &(dyn Fn(ImportProgress) + Sync),
) -> Result<ImportReport, AppError> {
    let path = checked_path(
        path,
        &["zip", "xml"],
        ".zip et .xml d'un export Apple Santé",
    )?;
    import_file(
        pool,
        SourceId::APPLE_HEALTH,
        cancel,
        on_progress,
        move |tx, cancel, shared| produce(&path, &limits, tx, cancel, shared),
    )
    .await
}
