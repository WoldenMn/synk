use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use sqlx::SqlitePool;
use synk_lib::error::AppError;
use synk_lib::sources::oauth::token::standard_json;
use synk_lib::sources::oauth::{ClientAuth, OAuthSpec};
use synk_lib::sources::sync::{sync_sources, Progress};
use synk_lib::sources::{
    Domain, FileSpec, Source, SourceId, SourceKind, SyncBatch, SyncContext, SyncFuture,
};

static OURA: OAuthSpec = OAuthSpec {
    id: SourceId::new("oura"),
    authorize_url: "",
    api_base: "",
    token_path: "",
    scopes: "",
    pkce: false,
    redirect_path: None,
    token_params: &[],
    authorize_params: &[],
    refresh_lifetime_secs: None,
    parse_token: standard_json,
    client_auth: ClientAuth::Body,
};

static POLAR: OAuthSpec = OAuthSpec {
    id: SourceId::new("polar"),
    authorize_url: "",
    api_base: "",
    token_path: "",
    scopes: "",
    pkce: false,
    redirect_path: None,
    token_params: &[],
    authorize_params: &[],
    refresh_lifetime_secs: None,
    parse_token: standard_json,
    client_auth: ClientAuth::Body,
};

enum Behavior {
    Writes(usize),
    SoftError(&'static str),
    Fails(&'static str),
}

struct TestSource {
    spec: Option<&'static OAuthSpec>,
    id: SourceId,
    label: &'static str,
    behavior: Behavior,
    runs: AtomicUsize,
}

impl TestSource {
    fn oauth(spec: &'static OAuthSpec, label: &'static str, behavior: Behavior) -> Self {
        Self {
            spec: Some(spec),
            id: spec.id,
            label,
            behavior,
            runs: AtomicUsize::new(0),
        }
    }
}

static TEST_FILE: FileSpec = FileSpec {
    extensions: &["fit"],
};

impl Source for TestSource {
    fn id(&self) -> SourceId {
        self.id
    }

    fn label(&self) -> &'static str {
        self.label
    }

    fn kind(&self) -> SourceKind {
        match self.spec {
            Some(spec) => SourceKind::OAuth(spec),
            None => SourceKind::File(&TEST_FILE),
        }
    }

    fn capabilities(&self) -> &'static [Domain] {
        &[Domain::Sleep]
    }

    fn steps(&self) -> u32 {
        2
    }

    fn sync<'a>(&'a self, ctx: &'a SyncContext<'a>) -> SyncFuture<'a> {
        Box::pin(async move {
            self.runs.fetch_add(1, Ordering::SeqCst);
            (ctx.on_progress)(1, "Sommeil");
            match &self.behavior {
                Behavior::Writes(n) => Ok(SyncBatch {
                    written: vec![(Domain::Sleep, *n)],
                    ..SyncBatch::default()
                }),
                Behavior::SoftError(msg) => Ok(SyncBatch {
                    written: vec![(Domain::Sleep, 1)],
                    errors: vec![(*msg).to_string()],
                    ..SyncBatch::default()
                }),
                Behavior::Fails(msg) => Err(AppError::Internal((*msg).to_string())),
            }
        })
    }
}

async fn connect(pool: &SqlitePool, source: &str) {
    sqlx::query(
        "INSERT INTO source_auth (source_id, access_token, refresh_token, expires_at) \
         VALUES (?1, 'a', 'r', 4102444800)",
    )
    .bind(source)
    .execute(pool)
    .await
    .expect("jeton");
}

async fn run(
    pool: &SqlitePool,
    sources: &[&dyn Source],
) -> (Vec<synk_lib::sources::sync::SourceOutcome>, Vec<String>) {
    let events = Mutex::new(Vec::new());
    let on_progress = |p: Progress| {
        events
            .lock()
            .expect("verrou")
            .push(format!("{}:{}/{}", p.source, p.etape, p.total));
    };
    let outcomes = sync_sources(sources, pool, &crate::common::test_vault(), &on_progress).await;
    let events = events.into_inner().expect("verrou");
    (outcomes, events)
}

#[tokio::test]
async fn a_failing_source_does_not_stop_the_next_one() {
    let pool = crate::common::memory_pool().await;
    connect(&pool, "oura").await;
    connect(&pool, "polar").await;
    let failing = TestSource::oauth(
        &OURA,
        "Oura",
        Behavior::Fails("refresh impossible, refresh_token=RT-FUITE-34"),
    );
    let healthy = TestSource::oauth(&POLAR, "Polar", Behavior::Writes(3));

    let (outcomes, _) = run(&pool, &[&failing, &healthy]).await;

    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].source, "oura");
    assert!(outcomes[0].failed);
    assert_eq!(outcomes[0].count, 0);
    assert!(
        outcomes[0].errors[0].starts_with("Oura : "),
        "{:?}",
        outcomes[0].errors
    );
    assert!(
        !outcomes[0].errors[0].contains("FUITE"),
        "{:?}",
        outcomes[0].errors
    );
    assert_eq!(healthy.runs.load(Ordering::SeqCst), 1);
    assert!(!outcomes[1].failed);
    assert_eq!(outcomes[1].count, 3);
    assert!(outcomes[1].errors.is_empty());
}

#[tokio::test]
async fn a_failure_is_logged_under_its_own_source_without_secrets() {
    let pool = crate::common::memory_pool().await;
    connect(&pool, "oura").await;
    connect(&pool, "polar").await;
    let failing = TestSource::oauth(
        &OURA,
        "Oura",
        Behavior::Fails("boom refresh_token=RT-FUITE-34"),
    );
    let healthy = TestSource::oauth(&POLAR, "Polar", Behavior::Writes(1));

    run(&pool, &[&failing, &healthy]).await;

    let rows: Vec<(String, String, Option<String>)> =
        sqlx::query_as("SELECT source, status, error FROM sync_log ORDER BY id")
            .fetch_all(&pool)
            .await
            .expect("journal");
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].0, "oura");
    assert_eq!(rows[0].1, "error");
    assert!(
        !rows[0].2.clone().unwrap_or_default().contains("FUITE"),
        "{rows:?}"
    );
}

#[tokio::test]
async fn a_soft_error_is_named_but_does_not_make_the_source_fail() {
    let pool = crate::common::memory_pool().await;
    connect(&pool, "oura").await;
    let partial = TestSource::oauth(&OURA, "Oura", Behavior::SoftError("Sommeil : 429"));

    let (outcomes, _) = run(&pool, &[&partial]).await;

    assert!(!outcomes[0].failed);
    assert_eq!(outcomes[0].count, 1);
    assert_eq!(outcomes[0].errors, vec!["Oura : Sommeil : 429".to_string()]);
}

#[tokio::test]
async fn a_source_that_is_not_connected_is_skipped() {
    let pool = crate::common::memory_pool().await;
    connect(&pool, "polar").await;
    let idle = TestSource::oauth(&OURA, "Oura", Behavior::Writes(9));
    let healthy = TestSource::oauth(&POLAR, "Polar", Behavior::Writes(2));
    let file = TestSource {
        spec: None,
        id: SourceId::new("csv"),
        label: "Fichiers CSV",
        behavior: Behavior::Writes(5),
        runs: AtomicUsize::new(0),
    };

    let (outcomes, _) = run(&pool, &[&idle, &file, &healthy]).await;

    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].source, "polar");
    assert_eq!(idle.runs.load(Ordering::SeqCst), 0);
    assert_eq!(file.runs.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn progress_names_the_source_and_its_step_count() {
    let pool = crate::common::memory_pool().await;
    connect(&pool, "oura").await;
    connect(&pool, "polar").await;
    let a = TestSource::oauth(&OURA, "Oura", Behavior::Writes(1));
    let b = TestSource::oauth(&POLAR, "Polar", Behavior::Writes(1));

    let (_, events) = run(&pool, &[&a, &b]).await;

    assert_eq!(
        events,
        vec![
            "oura:1/2".to_string(),
            "polar:1/2".to_string(),
            "analyses:1/1".to_string()
        ]
    );
}

async fn seed_a_weight_trend(pool: &SqlitePool) {
    for i in 0..8 {
        sqlx::query("INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES (strftime('%Y-%m-%dT08:00:00Z', 'now', ?1), 'manual', ?2)")
            .bind(format!("-{} days", 20 - i * 2))
            .bind(70.0 + f64::from(i))
            .execute(pool)
            .await
            .expect("pesee");
    }
}

async fn insight_count(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM insights")
        .fetch_one(pool)
        .await
        .expect("analyses")
}

#[tokio::test]
async fn analyses_run_once_after_every_source_and_not_once_per_source() {
    let pool = crate::common::memory_pool().await;
    connect(&pool, "oura").await;
    connect(&pool, "polar").await;
    seed_a_weight_trend(&pool).await;
    let a = TestSource::oauth(&OURA, "Oura", Behavior::Writes(1));
    let b = TestSource::oauth(&POLAR, "Polar", Behavior::Writes(1));

    let (outcomes, events) = run(&pool, &[&a, &b]).await;

    let analyses: Vec<usize> = events
        .iter()
        .enumerate()
        .filter(|(_, e)| e.starts_with("analyses:"))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(analyses, vec![events.len() - 1], "{events:?}");
    assert_eq!(insight_count(&pool).await, 1);
    assert_eq!(outcomes.len(), 2);
    assert!(outcomes.iter().all(|o| o.errors.is_empty() && !o.failed));
}

#[tokio::test]
async fn analyses_still_run_when_one_source_fails_but_another_succeeded() {
    let pool = crate::common::memory_pool().await;
    connect(&pool, "oura").await;
    connect(&pool, "polar").await;
    seed_a_weight_trend(&pool).await;
    let failing = TestSource::oauth(&OURA, "Oura", Behavior::Fails("panne"));
    let healthy = TestSource::oauth(&POLAR, "Polar", Behavior::Writes(1));

    let (_, events) = run(&pool, &[&failing, &healthy]).await;

    assert_eq!(events.last().map(String::as_str), Some("analyses:1/1"));
    assert_eq!(insight_count(&pool).await, 1);
}

#[tokio::test]
async fn no_analysis_runs_when_every_source_failed_or_none_was_due() {
    let pool = crate::common::memory_pool().await;
    connect(&pool, "oura").await;
    seed_a_weight_trend(&pool).await;
    let failing = TestSource::oauth(&OURA, "Oura", Behavior::Fails("panne"));
    let idle = TestSource::oauth(&POLAR, "Polar", Behavior::Writes(1));

    let (_, events) = run(&pool, &[&failing, &idle]).await;

    assert!(
        !events.iter().any(|e| e.starts_with("analyses:")),
        "{events:?}"
    );
    assert_eq!(insight_count(&pool).await, 0);

    let (outcomes, events) = run(&pool, &[]).await;
    assert!(outcomes.is_empty() && events.is_empty());
    assert_eq!(insight_count(&pool).await, 0);
}
