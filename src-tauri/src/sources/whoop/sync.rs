use std::sync::Arc;

use sqlx::SqlitePool;

use super::client::{WhoopClient, DAILY_LIMIT};
use super::mapping;
use super::persist;
use super::types::{Cycle, Recovery, Sleep, Workout};
use super::window::window_start;
use crate::error::AppError;
use crate::services::metrics::write_tolerant;
use crate::sources::oauth::clock::Clock;
use crate::sources::sync::logged_step;
use crate::sources::{store, Domain, Observation, SourceId, SyncBatch, SyncContext};

const QUOTA: &str = "quota";

#[derive(Default)]
struct StepOutcome {
    written: Vec<(Domain, usize)>,
    notes: Vec<String>,
}

#[derive(Clone, Copy)]
enum Kind {
    Sleep,
    Cycles,
    Workouts,
}

const STEPS: [(u32, &str, &str, Kind); 3] = [
    (1, "Sommeil et récupération", "sleep", Kind::Sleep),
    (2, "Cycles", "cycles", Kind::Cycles),
    (3, "Entraînements", "workouts", Kind::Workouts),
];

async fn write_vitals(
    pool: &SqlitePool,
    observations: Vec<Observation>,
    outcome: &mut StepOutcome,
) -> Result<(), AppError> {
    let report = write_tolerant(pool, &observations).await?;
    outcome.notes.extend(report.notes);
    outcome.written.push((Domain::Vitals, report.inserted));
    Ok(())
}

async fn sleep_step(
    client: &mut WhoopClient,
    pool: &SqlitePool,
    start: &str,
) -> Result<StepOutcome, AppError> {
    let sleeps: Vec<Sleep> = client.collection("activity/sleep", start).await?;
    let recoveries: Vec<Recovery> = client.collection("recovery", start).await?;
    let count = persist::write_sleep(pool, &mapping::sleep_rows(&sleeps)).await?;
    let mut outcome = StepOutcome::default();
    outcome.written.push((Domain::Sleep, count));
    let observations = mapping::recovery_observations(&recoveries, &sleeps);
    write_vitals(pool, observations, &mut outcome).await?;
    Ok(outcome)
}

async fn cycle_step(
    client: &mut WhoopClient,
    pool: &SqlitePool,
    start: &str,
) -> Result<StepOutcome, AppError> {
    let cycles: Vec<Cycle> = client.collection("cycle", start).await?;
    let (rows, observations) = mapping::cycle_rows(&cycles);
    let mut outcome = StepOutcome::default();
    let count = persist::write_activity(pool, &rows).await?;
    outcome.written.push((Domain::Activity, count));
    write_vitals(pool, observations, &mut outcome).await?;
    Ok(outcome)
}

async fn workout_step(
    client: &mut WhoopClient,
    pool: &SqlitePool,
    start: &str,
) -> Result<StepOutcome, AppError> {
    let workouts: Vec<Workout> = client.collection("activity/workout", start).await?;
    let count = persist::write_workouts(pool, &mapping::workout_rows(&workouts)).await?;
    Ok(StepOutcome {
        written: vec![(Domain::Workouts, count)],
        notes: Vec::new(),
    })
}

fn clock_time(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|t| t.with_timezone(&chrono::Local).format("%H:%M").to_string())
        .unwrap_or_default()
}

struct Run<'a> {
    pool: &'a SqlitePool,
    client: WhoopClient,
    batch: SyncBatch,
}

impl Run<'_> {
    async fn halt_on_limit(&mut self, why: &str) -> Result<(), AppError> {
        let text = match self.client.quota_resume() {
            Some(wait) => {
                let until = chrono::Utc::now().timestamp() + wait.as_secs() as i64;
                store::set_last_sync(self.pool, SourceId::WHOOP, QUOTA, until).await?;
                format!("{why}, reprise vers {}", clock_time(until))
            }
            None => format!("{why}, reprise a la prochaine synchro"),
        };
        self.batch.errors.push(format!("Whoop : {text}"));
        Ok(())
    }

    async fn step(
        &mut self,
        n: u32,
        label: &'static str,
        domain: &'static str,
        kind: Kind,
        ctx: &SyncContext<'_>,
    ) -> Result<bool, AppError> {
        (ctx.on_progress)(n, label);
        let pool = self.pool;
        let cursor = chrono::Utc::now();
        let last = store::get_last_sync(pool, SourceId::WHOOP, domain).await?;
        let start = window_start(last, cursor);
        let client = &mut self.client;
        let result = logged_step(
            pool,
            SourceId::WHOOP,
            domain,
            |o: &StepOutcome| o.written.iter().map(|(_, n)| *n as i64).sum(),
            || async {
                match kind {
                    Kind::Sleep => sleep_step(client, pool, &start).await,
                    Kind::Cycles => cycle_step(client, pool, &start).await,
                    Kind::Workouts => workout_step(client, pool, &start).await,
                }
            },
        )
        .await;
        match result {
            Ok(outcome) => {
                self.batch.written.extend(outcome.written);
                self.batch
                    .errors
                    .extend(outcome.notes.into_iter().map(|n| format!("{label} : {n}")));
                store::set_last_sync(pool, SourceId::WHOOP, domain, cursor.timestamp()).await?;
                Ok(true)
            }
            Err(e @ AppError::Unauthorized(_)) => Err(e),
            Err(AppError::RateLimit(why)) => {
                self.halt_on_limit(&why).await?;
                Ok(false)
            }
            Err(e) => {
                self.batch.errors.push(format!("{label} : {e}"));
                Ok(true)
            }
        }
    }
}

pub async fn run(
    base: &str,
    ctx: &SyncContext<'_>,
    per_minute: usize,
    clock: Arc<dyn Clock>,
) -> Result<SyncBatch, AppError> {
    let pool = ctx.pool;
    let mut batch = SyncBatch::default();
    let until = store::get_last_sync(pool, SourceId::WHOOP, QUOTA).await?;
    if until > chrono::Utc::now().timestamp() {
        batch.errors.push(format!(
            "Whoop : quota du jour atteint ({DAILY_LIMIT} requetes par jour), reprise vers {}",
            clock_time(until)
        ));
        return Ok(batch);
    }
    let client = WhoopClient::connect(base, pool, ctx.vault, clock, per_minute).await?;
    let mut run = Run {
        pool,
        client,
        batch,
    };
    for (n, label, domain, kind) in STEPS {
        if !run.step(n, label, domain, kind, ctx).await? {
            break;
        }
    }
    Ok(run.batch)
}
