use chrono::NaiveDate;
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;

use super::client::GoogleClient;
use super::mapping;
use super::persist;
use super::types::{
    ExercisePoint, HrvPoint, RespiratoryPoint, RestingHeartRatePoint, SleepPoint, Spo2Point,
    TemperaturePoint,
};
use super::window::{chunks, window, NARROW_ROLLUP_DAYS, WIDE_ROLLUP_DAYS};
use crate::error::AppError;
use crate::services::metrics::write_tolerant;
use crate::sources::sync::logged_step;
use crate::sources::{store, Domain, SourceId, SyncBatch, SyncContext};

const SESSION_PAGE: u32 = 25;
const DAILY_PAGE: u32 = 1000;

#[derive(Default)]
struct StepOutcome {
    written: Vec<(Domain, usize)>,
    notes: Vec<String>,
}

type Span = (NaiveDate, NaiveDate);

fn range_filter(field: &str, (start, end): Span) -> String {
    format!("{field} >= \"{start}\" AND {field} < \"{end}\"")
}

async fn daily<P: DeserializeOwned>(
    client: &mut GoogleClient,
    data_type: &str,
    span: Span,
) -> Result<Vec<P>, AppError> {
    let field = format!("{}.date", data_type.replace('-', "_"));
    client
        .list(data_type, &range_filter(&field, span), DAILY_PAGE)
        .await
}

async fn sleep_step(
    client: &mut GoogleClient,
    pool: &SqlitePool,
    span: Span,
) -> Result<StepOutcome, AppError> {
    let points: Vec<SleepPoint> = client
        .list(
            "sleep",
            &range_filter("sleep.interval.civil_end_time", span),
            SESSION_PAGE,
        )
        .await?;
    let breathing: Vec<RespiratoryPoint> = daily(client, "daily-respiratory-rate", span).await?;
    let rows = mapping::sleep_rows(&points, &breathing);
    let count = persist::write_sleep(pool, &rows).await?;
    Ok(StepOutcome {
        written: vec![(Domain::Sleep, count)],
        notes: Vec::new(),
    })
}

async fn activity_step(
    client: &mut GoogleClient,
    pool: &SqlitePool,
    span: Span,
) -> Result<StepOutcome, AppError> {
    let mut rollups = Vec::new();
    for (data_type, max_days) in [
        ("steps", WIDE_ROLLUP_DAYS),
        ("distance", WIDE_ROLLUP_DAYS),
        ("active-energy-burned", WIDE_ROLLUP_DAYS),
        ("total-calories", NARROW_ROLLUP_DAYS),
    ] {
        for (from, to) in chunks(span.0, span.1, max_days) {
            rollups.extend(client.daily_rollup(data_type, from, to).await?);
        }
    }
    let count = persist::write_activity(pool, &mapping::activity_rows(&rollups)).await?;
    Ok(StepOutcome {
        written: vec![(Domain::Activity, count)],
        notes: Vec::new(),
    })
}

async fn workout_step(
    client: &mut GoogleClient,
    pool: &SqlitePool,
    span: Span,
) -> Result<StepOutcome, AppError> {
    let points: Vec<ExercisePoint> = client
        .list(
            "exercise",
            &range_filter("exercise.interval.civil_start_time", span),
            SESSION_PAGE,
        )
        .await?;
    let count = persist::write_workouts(pool, &mapping::workout_rows(&points)).await?;
    Ok(StepOutcome {
        written: vec![(Domain::Workouts, count)],
        notes: Vec::new(),
    })
}

async fn vitals_step(
    client: &mut GoogleClient,
    pool: &SqlitePool,
    span: Span,
) -> Result<StepOutcome, AppError> {
    let resting: Vec<RestingHeartRatePoint> =
        daily(client, "daily-resting-heart-rate", span).await?;
    let hrv: Vec<HrvPoint> = daily(client, "daily-heart-rate-variability", span).await?;
    let spo2: Vec<Spo2Point> = daily(client, "daily-oxygen-saturation", span).await?;
    let temperature: Vec<TemperaturePoint> =
        daily(client, "daily-sleep-temperature-derivations", span).await?;
    let mut outcome = StepOutcome::default();
    let observations = mapping::daily_observations(&resting, &hrv, &spo2, &temperature);
    let report = write_tolerant(pool, &observations).await?;
    outcome.notes.extend(report.notes);
    outcome.written.push((Domain::Vitals, report.inserted));
    Ok(outcome)
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
            store::set_last_sync(pool, SourceId::GOOGLE_HEALTH, domain, cursor).await
        }
        Err(e @ AppError::Unauthorized(_)) => Err(e),
        Err(e) => {
            batch.errors.push(format!("{label} : {e}"));
            Ok(())
        }
    }
}

pub async fn run(
    token_base: &str,
    api_base: &str,
    ctx: &SyncContext<'_>,
) -> Result<SyncBatch, AppError> {
    let mut client = GoogleClient::connect(token_base, api_base, ctx.pool, ctx.vault).await?;
    client.verify_identity().await?;
    let mut batch = SyncBatch::default();
    let pool = ctx.pool;

    macro_rules! step {
        ($n:expr, $label:expr, $domain:expr, $run:ident) => {{
            (ctx.on_progress)($n, $label);
            let cursor = chrono::Utc::now();
            let last = store::get_last_sync(pool, SourceId::GOOGLE_HEALTH, $domain).await?;
            let span = window(last, cursor);
            let result = logged_step(
                pool,
                SourceId::GOOGLE_HEALTH,
                $domain,
                |o: &StepOutcome| o.written.iter().map(|(_, n)| *n as i64).sum(),
                || $run(&mut client, pool, span),
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
    step!(3, "Entraînements", "workouts", workout_step);
    step!(4, "Constantes", "vitals", vitals_step);
    Ok(batch)
}
