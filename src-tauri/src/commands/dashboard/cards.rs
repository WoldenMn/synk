use crate::commands::activity::Activity;
use crate::commands::blood_pressure::BloodPressure;
use crate::commands::body_measures::BodyMeasure;
use crate::commands::crud::{select_at, select_latest, CrudTable};
use crate::commands::heart::{latest_heart_record, HeartRecord};
use crate::commands::sleep::SleepRecord;
use crate::commands::stats::{check_trend_metric, metric_trend, TrendData};
use crate::commands::workouts::Workout;
use crate::error::AppError;
use crate::metric_repository::unknown_metric;
use crate::services::metrics::{load_tiebreak, resolve, Resolved, Tiebreak};
use crate::utils::dates::day_range;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct DashboardTrends {
    pub weight: TrendData,
    pub systolic: TrendData,
    pub sleep_score: TrendData,
    pub steps: TrendData,
}

#[derive(Debug, Serialize)]
pub struct DashboardCards {
    pub body: Option<BodyMeasure>,
    pub blood_pressure: Option<BloodPressure>,
    pub sleep: Option<SleepRecord>,
    pub activity: Option<Activity>,
    pub workout: Option<Workout>,
    pub heart: Option<HeartRecord>,
    pub trends: DashboardTrends,
}

async fn resolved(pool: &SqlitePool, slug: &str) -> Result<Resolved, AppError> {
    check_trend_metric(slug)?;
    resolve(pool, slug)
        .await?
        .ok_or_else(|| unknown_metric(slug))
}

async fn preferred_row<T: CrudTable>(
    pool: &SqlitePool,
    metric: &Resolved,
    tie: &Tiebreak,
) -> Result<Option<T>, AppError> {
    if let Some(point) = metric.latest(pool, tie).await? {
        if let Some(row) = select_at::<T>(pool, &point.measured_at, &point.source).await? {
            return Ok(Some(row));
        }
    }
    select_latest::<T>(pool).await
}

pub async fn dashboard_cards(
    pool: &SqlitePool,
    from: &str,
    to: &str,
) -> Result<DashboardCards, AppError> {
    day_range(from, to)?;
    let (tie, weight_m, systolic_m, sleep_m, steps_m) = tokio::try_join!(
        async { Ok(load_tiebreak(pool).await?) },
        resolved(pool, "weight"),
        resolved(pool, "systolic"),
        resolved(pool, "sleep_score"),
        resolved(pool, "steps"),
    )?;
    let (body, blood_pressure, sleep, activity, workout, heart) = tokio::try_join!(
        preferred_row::<BodyMeasure>(pool, &weight_m, &tie),
        preferred_row::<BloodPressure>(pool, &systolic_m, &tie),
        preferred_row::<SleepRecord>(pool, &sleep_m, &tie),
        preferred_row::<Activity>(pool, &steps_m, &tie),
        select_latest::<Workout>(pool),
        latest_heart_record(pool),
    )?;
    let (weight, systolic, sleep_score, steps) = tokio::try_join!(
        metric_trend(pool, &weight_m, &tie, from, to),
        metric_trend(pool, &systolic_m, &tie, from, to),
        metric_trend(pool, &sleep_m, &tie, from, to),
        metric_trend(pool, &steps_m, &tie, from, to),
    )?;
    Ok(DashboardCards {
        body,
        blood_pressure,
        sleep,
        activity,
        workout,
        heart,
        trends: DashboardTrends {
            weight,
            systolic,
            sleep_score,
            steps,
        },
    })
}

#[tauri::command]
pub async fn get_dashboard_cards(
    pool: State<'_, SqlitePool>,
    from: String,
    to: String,
) -> Result<DashboardCards, AppError> {
    dashboard_cards(pool.inner(), &from, &to).await
}
