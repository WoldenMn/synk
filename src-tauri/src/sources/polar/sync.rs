use sqlx::SqlitePool;

use super::client::PolarClient;
use super::mapping;
use super::persist;
use super::types::{Activity, Exercise, Nights, Recharges};
use super::window::activity_ranges;
use crate::error::AppError;
use crate::services::metrics::write_tolerant;
use crate::sources::sync::logged_step;
use crate::sources::{store, Domain, SourceId, SyncBatch, SyncContext};

const REGISTRATION: &str = "registration";

#[derive(Default)]
struct StepOutcome {
    written: Vec<(Domain, usize)>,
    notes: Vec<String>,
}

async fn ensure_registered(client: &mut PolarClient, pool: &SqlitePool) -> Result<(), AppError> {
    if store::get_last_sync(pool, SourceId::POLAR, REGISTRATION).await? > 0 {
        return Ok(());
    }
    client.register().await?;
    store::set_last_sync(
        pool,
        SourceId::POLAR,
        REGISTRATION,
        chrono::Utc::now().timestamp(),
    )
    .await
}

async fn sleep_step(client: &mut PolarClient, pool: &SqlitePool) -> Result<StepOutcome, AppError> {
    let nights = client
        .get::<Nights>("/v3/users/sleep")
        .await?
        .map(|n| n.nights)
        .unwrap_or_default();
    let recharges = client
        .get::<Recharges>("/v3/users/nightly-recharge")
        .await?
        .map(|r| r.recharges)
        .unwrap_or_default();
    let rows = mapping::sleep_rows(&nights, &recharges);
    let mut outcome = StepOutcome::default();
    outcome
        .written
        .push((Domain::Sleep, persist::write_sleep(pool, &rows).await?));
    let report = write_tolerant(pool, &mapping::recharge_observations(&recharges)).await?;
    outcome.notes.extend(report.notes);
    outcome.written.push((Domain::Vitals, report.inserted));
    Ok(outcome)
}

async fn activity_step(
    client: &mut PolarClient,
    pool: &SqlitePool,
) -> Result<StepOutcome, AppError> {
    let last = store::get_last_sync(pool, SourceId::POLAR, "activity").await?;
    let mut all: Vec<Activity> = Vec::new();
    for (from, to) in activity_ranges(last, chrono::Utc::now()) {
        let path = format!("/v3/users/activities?from={from}&to={to}");
        all.extend(
            client
                .get::<Vec<Activity>>(&path)
                .await?
                .unwrap_or_default(),
        );
    }
    let count = persist::write_activity(pool, &mapping::activity_rows(&all)).await?;
    Ok(StepOutcome {
        written: vec![(Domain::Activity, count)],
        notes: Vec::new(),
    })
}

async fn workout_step(
    client: &mut PolarClient,
    pool: &SqlitePool,
) -> Result<StepOutcome, AppError> {
    let docs = client
        .get::<Vec<Exercise>>("/v3/exercises")
        .await?
        .unwrap_or_default();
    let count = persist::write_workouts(pool, &mapping::workout_rows(&docs)).await?;
    Ok(StepOutcome {
        written: vec![(Domain::Workouts, count)],
        notes: Vec::new(),
    })
}

async fn settle(
    pool: &SqlitePool,
    batch: &mut SyncBatch,
    label: &str,
    domain: &str,
    cursor: i64,
    result: Result<StepOutcome, AppError>,
) -> Result<(), AppError> {
    match result {
        Ok(outcome) => {
            batch.written.extend(outcome.written);
            batch
                .errors
                .extend(outcome.notes.into_iter().map(|n| format!("{label} : {n}")));
            store::set_last_sync(pool, SourceId::POLAR, domain, cursor).await
        }
        Err(e @ AppError::Unauthorized(_)) => Err(e),
        Err(e) => {
            batch.errors.push(format!("{label} : {e}"));
            Ok(())
        }
    }
}

pub async fn run(base: &str, ctx: &SyncContext<'_>) -> Result<SyncBatch, AppError> {
    let mut client = PolarClient::connect(base, ctx.pool, ctx.vault).await?;
    ensure_registered(&mut client, ctx.pool).await?;
    let mut batch = SyncBatch::default();
    let pool = ctx.pool;

    macro_rules! step {
        ($n:expr, $label:expr, $domain:expr, $run:ident) => {{
            (ctx.on_progress)($n, $label);
            let cursor = chrono::Utc::now().timestamp();
            let result = logged_step(
                pool,
                SourceId::POLAR,
                $domain,
                |o: &StepOutcome| o.written.iter().map(|(_, n)| *n as i64).sum(),
                || $run(&mut client, pool),
            )
            .await;
            settle(pool, &mut batch, $label, $domain, cursor, result).await?;
        }};
    }

    step!(1, "Sommeil", "sleep", sleep_step);
    step!(2, "Activité", "activity", activity_step);
    step!(3, "Entraînements", "workouts", workout_step);
    Ok(batch)
}
