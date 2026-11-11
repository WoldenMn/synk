use sqlx::SqlitePool;

use super::client::OuraClient;
use super::mapping;
use super::persist;
use super::types::{DailyActivity, DailyReadiness, DailySleep, DailySpo2, SleepPeriod, Workout};
use super::window::window;
use crate::error::AppError;
use crate::services::metrics::write_tolerant;
use crate::sources::sync::logged_step;
use crate::sources::{store, Domain, Observation, SourceId, SyncBatch, SyncContext};

#[derive(Default)]
struct StepOutcome {
    written: Vec<(Domain, usize)>,
    notes: Vec<String>,
}

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
    client: &mut OuraClient,
    pool: &SqlitePool,
    start: &str,
    end: &str,
) -> Result<StepOutcome, AppError> {
    let periods: Vec<SleepPeriod> = client.collection("sleep", start, end).await?;
    let scores: Vec<DailySleep> = client.collection("daily_sleep", start, end).await?;
    let rows = mapping::sleep_rows(&periods, &scores);
    let mut outcome = StepOutcome::default();
    let count = persist::write_sleep(pool, &rows).await?;
    outcome.written.push((Domain::Sleep, count));
    write_vitals(pool, mapping::sleep_observations(&rows), &mut outcome).await?;
    Ok(outcome)
}

async fn activity_step(
    client: &mut OuraClient,
    pool: &SqlitePool,
    start: &str,
    end: &str,
) -> Result<StepOutcome, AppError> {
    let docs: Vec<DailyActivity> = client.collection("daily_activity", start, end).await?;
    let count = persist::write_activity(pool, &mapping::activity_rows(&docs)).await?;
    Ok(StepOutcome {
        written: vec![(Domain::Activity, count)],
        notes: Vec::new(),
    })
}

async fn recovery_step(
    client: &mut OuraClient,
    pool: &SqlitePool,
    start: &str,
    end: &str,
) -> Result<StepOutcome, AppError> {
    let readiness: Vec<DailyReadiness> = client.collection("daily_readiness", start, end).await?;
    let spo2: Vec<DailySpo2> = client.collection("daily_spo2", start, end).await?;
    let mut observations = mapping::readiness_observations(&readiness);
    observations.extend(mapping::spo2_observations(&spo2));
    let mut outcome = StepOutcome::default();
    write_vitals(pool, observations, &mut outcome).await?;
    Ok(outcome)
}

async fn workout_step(
    client: &mut OuraClient,
    pool: &SqlitePool,
    start: &str,
    end: &str,
) -> Result<StepOutcome, AppError> {
    let docs: Vec<Workout> = client.collection("workout", start, end).await?;
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
            store::set_last_sync(pool, SourceId::OURA, domain, cursor).await
        }
        Err(e @ AppError::Unauthorized(_)) => Err(e),
        Err(e) => {
            batch.errors.push(format!("{label} : {e}"));
            Ok(())
        }
    }
}

pub async fn run(base: &str, ctx: &SyncContext<'_>) -> Result<SyncBatch, AppError> {
    let mut client = OuraClient::connect(base, ctx.pool, ctx.vault).await?;
    let mut batch = SyncBatch::default();
    let pool = ctx.pool;

    macro_rules! step {
        ($n:expr, $label:expr, $domain:expr, $run:ident) => {{
            (ctx.on_progress)($n, $label);
            let cursor = chrono::Utc::now();
            let last = store::get_last_sync(pool, SourceId::OURA, $domain).await?;
            let (start, end) = window(last, cursor);
            let result = logged_step(
                pool,
                SourceId::OURA,
                $domain,
                |o: &StepOutcome| o.written.iter().map(|(_, n)| *n as i64).sum(),
                || $run(&mut client, pool, &start, &end),
            )
            .await;
            settle(
                pool,
                &mut batch,
                $label,
                $domain,
                cursor.timestamp(),
                result,
            )
            .await?;
        }};
    }

    step!(1, "Sommeil", "sleep", sleep_step);
    step!(2, "Activité", "activity", activity_step);
    step!(3, "Récupération", "readiness", recovery_step);
    step!(4, "Entraînements", "workouts", workout_step);
    Ok(batch)
}
