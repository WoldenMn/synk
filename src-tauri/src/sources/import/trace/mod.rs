pub mod gpx;
pub mod row;
pub mod source;
pub mod stats;
pub mod tcx;
pub mod xml;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use sqlx::SqlitePool;

use super::limits::Limits;
use super::pipeline::{
    checked_path, file_size, import_file, send_all, ImportProgress, ImportReport, Produced, Sender,
    Shared,
};
use super::rows::{Out, Skipped, WorkoutRow};
use crate::error::AppError;
use crate::sources::SourceId;

pub use source::TRACE;

const BUFFER_BYTES: usize = 256 * 1024;

#[derive(Debug, Default)]
pub struct Parsed {
    pub rows: Vec<WorkoutRow>,
    pub skipped: Skipped,
    pub cancelled: bool,
}

fn produce(
    path: &Path,
    tx: &Sender,
    cancel: &AtomicBool,
    shared: &Shared,
) -> Result<Produced, AppError> {
    file_size(path, shared)?;
    Limits::TRACE.check_size(
        shared.size.load(std::sync::atomic::Ordering::Relaxed),
        "Fichier GPX ou TCX",
    )?;
    let file =
        File::open(path).map_err(|e| AppError::Validation(format!("fichier illisible : {e}")))?;
    let reader = BufReader::with_capacity(BUFFER_BYTES, file);
    let is_gpx = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("gpx"));
    let parsed = if is_gpx {
        gpx::parse(reader, cancel)?
    } else {
        tcx::parse(reader, cancel)?
    };
    if parsed.cancelled {
        return Ok(Produced {
            cancelled: true,
            skipped: Skipped::default(),
        });
    }
    send_all(tx, parsed.rows.into_iter().map(Out::Workout).collect())?;
    Ok(Produced {
        cancelled: false,
        skipped: parsed.skipped,
    })
}

pub async fn import_trace(
    pool: &SqlitePool,
    path: &str,
    cancel: Arc<AtomicBool>,
    on_progress: &(dyn Fn(ImportProgress) + Sync),
) -> Result<ImportReport, AppError> {
    let path = checked_path(path, &["gpx", "tcx"], ".gpx et .tcx")?;
    import_file(
        pool,
        SourceId::TRACE,
        cancel,
        on_progress,
        move |tx, cancel, shared| produce(&path, tx, cancel, shared),
    )
    .await
}
