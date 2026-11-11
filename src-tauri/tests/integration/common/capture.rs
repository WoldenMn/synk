use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::fmt::Debug;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, Once};
use tracing::field::{Field, Visit};
use tracing::{span, Event, Metadata, Subscriber};

static EVENTS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
static INSTALL: Once = Once::new();
static MARKERS: AtomicUsize = AtomicUsize::new(0);

struct Summary(String);

impl Visit for Summary {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "summary" {
            self.0 = format!("{value:?}");
        }
    }
}

struct Statements;

impl Subscriber for Statements {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.target() == "sqlx::query"
    }

    fn new_span(&self, _: &span::Attributes<'_>) -> span::Id {
        span::Id::from_u64(1)
    }

    fn record(&self, _: &span::Id, _: &span::Record<'_>) {}

    fn record_follows_from(&self, _: &span::Id, _: &span::Id) {}

    fn event(&self, event: &Event<'_>) {
        if event.metadata().target() != "sqlx::query" {
            return;
        }
        let mut summary = Summary(String::new());
        event.record(&mut summary);
        let thread = std::thread::current().name().unwrap_or("").to_string();
        EVENTS.lock().unwrap().push((thread, summary.0));
    }

    fn enter(&self, _: &span::Id) {}

    fn exit(&self, _: &span::Id) {}
}

pub struct CountedPool {
    pub pool: SqlitePool,
    worker: String,
}

pub async fn counted_pool() -> CountedPool {
    INSTALL.call_once(|| {
        tracing::subscriber::set_global_default(Statements)
            .expect("aucun autre abonne tracing dans ce binaire");
    });
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();
    synk_lib::db::bootstrap_schema(&pool).await.unwrap();
    let marker = format!("SELECT 'worker-{}'", MARKERS.fetch_add(1, Ordering::SeqCst));
    sqlx::query(sqlx::AssertSqlSafe(marker.clone()))
        .execute(&pool)
        .await
        .unwrap();
    let worker = EVENTS
        .lock()
        .unwrap()
        .iter()
        .rev()
        .find(|(_, summary)| summary.contains(&marker))
        .map(|(thread, _)| thread.clone())
        .expect("l'instrument ne voit pas la requete repere");
    CountedPool { pool, worker }
}

impl CountedPool {
    fn seen(&self) -> usize {
        EVENTS
            .lock()
            .unwrap()
            .iter()
            .filter(|(thread, _)| *thread == self.worker)
            .count()
    }

    pub async fn count<F: Future>(&self, work: F) -> (F::Output, usize) {
        let before = self.seen();
        let output = work.await;
        (output, self.seen() - before)
    }
}
