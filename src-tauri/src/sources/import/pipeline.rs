use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use sqlx::SqlitePool;
use tokio::sync::mpsc;

use super::persist::{Persister, Tally};
use super::rows::{Out, Skipped};
use crate::error::AppError;
use crate::sources::sync::logged_step;
use crate::sources::SourceId;

pub const BATCH: usize = 2000;
const TICK: Duration = Duration::from_millis(200);
const QUEUE: usize = 4;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImportProgress {
    pub source: &'static str,
    pub percent: u8,
    pub records: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SkippedItem {
    pub reason: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImportReport {
    pub source: &'static str,
    pub cancelled: bool,
    pub tally: Tally,
    pub skipped: Vec<SkippedItem>,
    pub duration_ms: u64,
}

pub struct Shared {
    pub seen: Arc<AtomicU64>,
    pub size: AtomicU64,
}

pub struct Produced {
    pub cancelled: bool,
    pub skipped: Skipped,
}

pub type Sender = mpsc::Sender<Vec<Out>>;

pub fn checked_path(path: &str, allowed: &[&str], what: &str) -> Result<PathBuf, AppError> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| AppError::Validation(format!("Chemin invalide : {e}")))?;
    let ok = canonical
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| allowed.contains(&e.to_lowercase().as_str()));
    if !ok {
        return Err(AppError::Validation(format!(
            "Seuls les fichiers {what} sont acceptés"
        )));
    }
    Ok(canonical)
}

pub fn send_all(tx: &Sender, outs: Vec<Out>) -> Result<(), AppError> {
    let closed = || AppError::Internal("import interrompu".into());
    let mut outs = outs.into_iter().peekable();
    while outs.peek().is_some() {
        let chunk: Vec<Out> = outs.by_ref().take(BATCH).collect();
        tx.blocking_send(chunk).map_err(|_| closed())?;
    }
    Ok(())
}

fn percent(shared: &Shared) -> u8 {
    let size = shared.size.load(Ordering::Relaxed).max(1);
    let seen = shared.seen.load(Ordering::Relaxed);
    (seen.saturating_mul(100) / size).min(99) as u8
}

async fn consume(
    pool: &SqlitePool,
    source: SourceId,
    rx: &mut mpsc::Receiver<Vec<Out>>,
    cancel: &AtomicBool,
    shared: &Shared,
    on_progress: &(dyn Fn(ImportProgress) + Sync),
) -> Result<(sqlx::Transaction<'static, sqlx::Sqlite>, Persister), AppError> {
    let mut tx = pool.begin().await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "PRAGMA busy_timeout={}",
        crate::db::gate::BUSY_TIMEOUT_MS
    )))
    .execute(&mut *tx)
    .await?;
    let mut persister = Persister::new(source);
    let mut tick = tokio::time::interval(TICK);
    loop {
        tokio::select! {
            batch = rx.recv() => {
                let Some(batch) = batch else { break };
                for out in batch {
                    persister.write(&mut tx, out).await?;
                }
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
            }
            _ = tick.tick() => on_progress(ImportProgress {
                source: source.as_str(),
                percent: percent(shared),
                records: persister.tally.written(),
            }),
        }
    }
    Ok((tx, persister))
}

pub async fn run_import<P>(
    pool: &SqlitePool,
    source: SourceId,
    cancel: Arc<AtomicBool>,
    on_progress: &(dyn Fn(ImportProgress) + Sync),
    produce: P,
) -> Result<ImportReport, AppError>
where
    P: FnOnce(&Sender, &AtomicBool, &Shared) -> Result<Produced, AppError> + Send + 'static,
{
    let clock = Instant::now();
    let _long_write = crate::db::gate::LongWrite::begin();
    let (sender, mut rx) = mpsc::channel::<Vec<Out>>(QUEUE);
    let shared = Arc::new(Shared {
        seen: Arc::new(AtomicU64::new(0)),
        size: AtomicU64::new(0),
    });
    let producer = {
        let (cancel, shared) = (Arc::clone(&cancel), Arc::clone(&shared));
        tokio::task::spawn_blocking(move || produce(&sender, &cancel, &shared))
    };
    let consumed = consume(pool, source, &mut rx, &cancel, &shared, on_progress).await;
    if consumed.is_err() {
        cancel.store(true, Ordering::Relaxed);
    }
    drop(rx);
    let produced = producer
        .await
        .map_err(|e| AppError::Internal(format!("lecture de l'import interrompue : {e}")))?;
    let (tx, mut persister) = consumed?;
    let produced = produced?;
    let stopped = produced.cancelled || cancel.load(Ordering::Relaxed);
    let report = |cancelled: bool, tally: Tally, skipped: Skipped| ImportReport {
        source: source.as_str(),
        cancelled,
        tally,
        skipped: skipped
            .0
            .into_iter()
            .map(|(reason, count)| SkippedItem { reason, count })
            .collect(),
        duration_ms: clock.elapsed().as_millis() as u64,
    };
    if stopped {
        tx.rollback().await?;
        return Ok(report(true, Tally::default(), Skipped::default()));
    }
    tx.commit().await?;
    for (reason, count) in produced.skipped.0 {
        persister.skipped.add_many(reason, count);
    }
    on_progress(ImportProgress {
        source: source.as_str(),
        percent: 100,
        records: persister.tally.written(),
    });
    Ok(report(false, persister.tally, persister.skipped))
}

pub async fn import_file<P>(
    pool: &SqlitePool,
    source: SourceId,
    cancel: Arc<AtomicBool>,
    on_progress: &(dyn Fn(ImportProgress) + Sync),
    produce: P,
) -> Result<ImportReport, AppError>
where
    P: FnOnce(&Sender, &AtomicBool, &Shared) -> Result<Produced, AppError> + Send + 'static,
{
    logged_step(
        pool,
        source,
        "import",
        |r: &ImportReport| r.tally.written() as i64,
        || run_import(pool, source, cancel, on_progress, produce),
    )
    .await
}

pub fn file_size(path: &Path, shared: &Shared) -> Result<(), AppError> {
    let size = std::fs::metadata(path)
        .map_err(|e| AppError::Validation(format!("fichier illisible : {e}")))?
        .len();
    shared.size.store(size, Ordering::Relaxed);
    shared.seen.store(size, Ordering::Relaxed);
    Ok(())
}
