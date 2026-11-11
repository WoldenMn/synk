use synk_lib::commands::body_measures::BodyMeasure;
use synk_lib::commands::crud::select_range;
use synk_lib::commands::heart::heart_records_between;
use synk_lib::commands::labs::lab_results_between;
use synk_lib::commands::manual_measurements::{
    get_measurement_overview_inner, get_measurement_stats_inner, manual_measurements_between,
};
use synk_lib::commands::medications::medication_logs_between;
use synk_lib::commands::stats::run_metric_trend;
use synk_lib::error::AppError;
use synk_lib::metric_repository::read_metric_series;
use synk_lib::services::metrics::{select_secondary, select_series};
use synk_lib::utils::dates::day_range;

const BAD: &str = "pas-une-date";

fn is_validation<T: std::fmt::Debug>(result: Result<T, AppError>, what: &str) {
    match result {
        Err(AppError::Validation(message)) => {
            assert!(
                message.contains(BAD),
                "{what} : message sans la valeur ({message})"
            )
        }
        other => panic!("{what} : Validation attendue, obtenu {other:?}"),
    }
}

#[tokio::test]
async fn an_unreadable_from_bound_is_refused_by_every_range_read() {
    let pool = crate::common::memory_pool().await;
    let to = "2026-04-30";

    is_validation(
        select_range::<BodyMeasure>(&pool, BAD.into(), to.into()).await,
        "crud",
    );
    is_validation(run_metric_trend(&pool, "weight", BAD, to).await, "stats");
    is_validation(
        read_metric_series(&pool, "weight", BAD, to).await,
        "metric_repository",
    );
    is_validation(
        select_series(&pool, "weight", BAD, to).await,
        "select_series",
    );
    is_validation(
        select_secondary(&pool, "weight", BAD, to).await,
        "select_secondary",
    );
    is_validation(heart_records_between(&pool, BAD, to).await, "heart");
    is_validation(lab_results_between(&pool, BAD, to).await, "labs");
    is_validation(
        medication_logs_between(&pool, 1, BAD, to).await,
        "medications",
    );
    is_validation(
        manual_measurements_between(&pool, 1, BAD, to).await,
        "measurements",
    );
    is_validation(
        get_measurement_stats_inner(&pool, 1, BAD, to).await,
        "stats mensurations",
    );
    is_validation(
        get_measurement_overview_inner(&pool, BAD, to).await,
        "apercu",
    );
}

#[tokio::test]
async fn an_unreadable_to_bound_is_still_refused() {
    let pool = crate::common::memory_pool().await;
    let result = read_metric_series(&pool, "weight", "2026-04-01", BAD).await;
    assert!(matches!(result, Err(AppError::Validation(_))), "{result:?}");
}

#[tokio::test]
async fn a_bound_with_a_time_and_an_offset_is_still_read_by_its_day() {
    let pool = crate::common::memory_pool().await;
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, weight, source) \
         VALUES ('2026-04-10T08:00:00Z', 73.0, 'manual')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let points = select_series(
        &pool,
        "weight",
        "2026-04-01T00:00:00+02:00",
        "2026-04-30T23:59:59Z",
    )
    .await
    .unwrap();
    assert_eq!(points.len(), 1);
}

#[tokio::test]
async fn an_unknown_slug_is_not_found_and_not_an_empty_list() {
    let pool = crate::common::memory_pool().await;
    for result in [
        read_metric_series(&pool, "inconnue", "2026-04-01", "2026-04-30").await,
        select_series(&pool, "inconnue", "2026-04-01", "2026-04-30").await,
        select_secondary(&pool, "inconnue", "2026-04-01", "2026-04-30").await,
    ] {
        match result {
            Err(error @ AppError::NotFoundByKey { .. }) => {
                assert_eq!(error.kind(), "NotFound");
                assert!(error.detail().contains("inconnue"), "{}", error.detail());
            }
            other => panic!("NotFound attendue, obtenu {other:?}"),
        }
    }
}

#[tokio::test]
async fn a_known_slug_without_data_is_still_an_empty_list() {
    let pool = crate::common::memory_pool().await;
    let points = select_series(&pool, "weight", "2026-04-01", "2026-04-30")
        .await
        .unwrap();
    assert!(points.is_empty());
}

#[test]
fn day_range_gives_the_first_day_and_the_day_after_the_last() {
    let range = day_range("2026-04-01T00:00:00+02:00", "2026-04-30T12:00:00Z").unwrap();
    assert_eq!(range.from, "2026-04-01");
    assert_eq!(range.before, "2026-05-01");
    assert!(day_range("", "2026-04-30").is_err());
}

#[tokio::test]
async fn the_exports_refuse_an_unreadable_from_bound() {
    let pool = crate::common::memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let categories = vec!["body_measures".to_string()];
    let csv = synk_lib::commands::export::export_csv_inner(
        &pool,
        BAD,
        "2026-04-30",
        &categories,
        &dir.path().join("a.csv"),
    )
    .await;
    is_validation(csv, "export csv");
    let pdf = synk_lib::commands::export::export_pdf_to(
        &pool,
        BAD,
        "2026-04-30",
        &categories,
        &dir.path().join("a.pdf"),
    )
    .await;
    is_validation(pdf, "export pdf");
}
