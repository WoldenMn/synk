use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

use sqlx::SqlitePool;
use synk_lib::services::metrics::{select_secondary, select_series};
use synk_lib::sources::import::apple_health::mapping::{wants, Mapper};
use synk_lib::sources::import::apple_health::open::with_export;
use synk_lib::sources::import::apple_health::reader::parse_export;
use synk_lib::sources::import::apple_health::run::BATCH;
use synk_lib::sources::import::apple_health::{import_apple_health, ImportReport};

use crate::common::memory_pool;
use crate::support::apple_export::{record, small_export, write_xml, zip_of};
use crate::support::import_gate;
use crate::support::peak_alloc::peak_of;

const PEAK_CEILING_BYTES: i64 = 2 * 1024 * 1024;

const SLEEP: &str = r#" <Record type="HKCategoryTypeIdentifierSleepAnalysis" sourceName="Apple Watch" startDate="2026-10-25 23:30:00 +0200" endDate="2026-10-26 02:30:00 +0200" value="HKCategoryValueSleepAnalysisAsleepCore"/>
 <Record type="HKCategoryTypeIdentifierSleepAnalysis" sourceName="Apple Watch" startDate="2026-10-26 02:30:00 +0200" endDate="2026-10-26 04:00:00 +0200" value="HKCategoryValueSleepAnalysisAsleepDeep"/>
 <Record type="HKCategoryTypeIdentifierSleepAnalysis" sourceName="Apple Watch" startDate="2026-10-26 04:00:00 +0200" endDate="2026-10-26 05:00:00 +0200" value="HKCategoryValueSleepAnalysisAsleepREM"/>
"#;

const WORKOUT: &str = r#" <Workout workoutActivityType="HKWorkoutActivityTypeRunning" duration="30.5" durationUnit="min" totalDistance="3.1" totalDistanceUnit="mi" totalEnergyBurned="320" totalEnergyBurnedUnit="kcal" sourceName="Apple Watch" startDate="2026-10-25 18:00:00 +0200" endDate="2026-10-25 18:30:30 +0200">
  <WorkoutEvent type="HKWorkoutEventTypePause" date="2026-10-25 18:10:00 +0200"/>
 </Workout>
"#;

fn day_body() -> String {
    let mut body = String::new();
    let morning = "2026-10-25 07:30:00 +0200";
    let evening = "2026-10-25 23:30:00 +0200";
    body += &record("BodyMass", "lb", "160", "Withings", morning);
    body += &record("BodyMass", "kg", "72.0", "Balance", evening);
    body += &record("OxygenSaturation", "%", "0.97", "Oura", morning);
    body += &record(
        "BloodGlucose",
        "mmol&lt;180.1558800000541&gt;/L",
        "5.5",
        "Libre",
        morning,
    );
    body += &record("BloodPressureSystolic", "mmHg", "121", "Tensio", morning);
    body += &record("BloodPressureDiastolic", "mmHg", "79", "Tensio", morning);
    body += &record("StepCount", "count", "4000", "iPhone", morning);
    body += &record("StepCount", "count", "6100", "Apple Watch", evening);
    body += &record("DistanceWalkingRunning", "mi", "3", "Apple Watch", morning);
    body += &record("ActiveEnergyBurned", "kJ", "1716.4", "Apple Watch", morning);
    body += &record("BasalEnergyBurned", "kcal", "1700", "Apple Watch", morning);
    body += &record("HeartRate", "count/min", "62", "Apple Watch", morning);
    body += SLEEP;
    body += WORKOUT;
    body
}

async fn import(pool: &SqlitePool, path: &Path) -> ImportReport {
    import_apple_health(
        pool,
        path.to_str().unwrap(),
        Arc::new(AtomicBool::new(false)),
        &|_| {},
    )
    .await
    .expect("import")
}

async fn count(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT COUNT(*) FROM {table} WHERE source = 'apple_health'"
    )))
    .fetch_one(pool)
    .await
    .unwrap()
}

const TABLES: [&str; 6] = [
    "body_measures_legacy",
    "vitals_legacy",
    "blood_pressure",
    "sleep_records",
    "activity",
    "workouts",
];

async fn counts(pool: &SqlitePool) -> Vec<i64> {
    let mut all = Vec::new();
    for t in TABLES {
        all.push(count(pool, t).await);
    }
    all
}

#[tokio::test]
async fn a_day_of_data_lands_in_the_right_tables_with_converted_units_and_origin_devices() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let report = import(&pool, &small_export(dir.path(), &day_body())).await;

    assert!(!report.cancelled);
    assert_eq!(counts(&pool).await, vec![2, 2, 1, 1, 1, 1]);
    assert_eq!(report.tally.devices.get("Withings"), Some(&1));

    let weights: Vec<(String, f64, Option<String>)> = sqlx::query_as(
        "SELECT measured_at, weight, device FROM body_measures_legacy WHERE source = 'apple_health' ORDER BY measured_at",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(weights[0].0, "2026-10-25T05:30:00Z");
    assert!((weights[0].1 - 72.5748).abs() < 1e-3);
    assert_eq!(weights[0].2.as_deref(), Some("Withings"));
    assert_eq!(weights[1].0, "2026-10-25T21:30:00Z");

    let spo2: (f64, Option<String>) = sqlx::query_as(
        "SELECT value, device FROM vitals_legacy WHERE vital_type = 'spo2' AND source = 'apple_health'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!((spo2.0 - 97.0).abs() < 1e-9);
    assert_eq!(spo2.1.as_deref(), Some("Oura"));

    let glucose: f64 =
        sqlx::query_scalar("SELECT value FROM vitals_legacy WHERE vital_type = 'blood_glucose'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!((glucose - 5.5).abs() < 1e-9);

    let bp: (i64, i64, Option<String>) = sqlx::query_as(
        "SELECT systolic, diastolic, device FROM blood_pressure WHERE source = 'apple_health'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((bp.0, bp.1, bp.2.as_deref()), (121, 79, Some("Tensio")));

    let activity: (String, i64, f64, f64, f64, Option<String>) = sqlx::query_as(
        "SELECT date, steps, distance, calories_active, calories_total, device FROM activity WHERE source = 'apple_health'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(activity.0, "2026-10-25");
    assert_eq!(activity.1, 6100);
    assert!((activity.2 - 4828.032).abs() < 1e-3);
    assert!((activity.3 - 410.2).abs() < 0.1);
    assert!((activity.4 - 2110.2).abs() < 0.1);
    assert_eq!(activity.5.as_deref(), Some("Apple Watch"));

    let sleep: (String, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT date, duration_total, duration_light, duration_deep, duration_rem FROM sleep_records WHERE source = 'apple_health'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        sleep,
        ("2026-10-26".to_string(), 19_800, 10_800, 5_400, 3_600)
    );

    let workout: (String, String, i64, f64, f64) = sqlx::query_as(
        "SELECT date, category_label, duration, distance, calories FROM workouts WHERE source = 'apple_health'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(workout.0, "2026-10-25");
    assert_eq!(workout.1, "Course");
    assert_eq!(workout.2, 1830);
    assert!((workout.3 - 4988.97).abs() < 0.1);
}

#[tokio::test]
async fn importing_the_same_export_twice_adds_nothing() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = small_export(dir.path(), &day_body());
    import(&pool, &path).await;
    let first = counts(&pool).await;

    let again = import(&pool, &path).await;

    assert_eq!(counts(&pool).await, first);
    assert_eq!(again.tally.observations, 0);
    assert_eq!(again.tally.blood_pressure, 0);
    assert_eq!(again.tally.workouts, 0);
    assert_eq!(again.tally.sleep, 0);
    assert_eq!(again.tally.activity, 0);
    assert_eq!(again.tally.observations_ignored, 4);
}

#[tokio::test]
async fn a_direct_withings_weight_and_the_apple_copy_of_it_show_one_value_a_day() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, weight) VALUES ('2026-10-25T05:30:00Z', 'withings', 72.57)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let body = record(
        "BodyMass",
        "lb",
        "160",
        "Withings",
        "2026-10-25 07:30:00 +0200",
    );
    import(&pool, &small_export(dir.path(), &body)).await;

    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM body_measures_legacy")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 2);
    let series = select_series(&pool, "weight", "2026-10-01", "2026-10-31")
        .await
        .unwrap();
    assert_eq!(series.len(), 1, "{series:?}");
    assert_eq!(series[0].source, "withings");
    let secondary = select_secondary(&pool, "weight", "2026-10-01", "2026-10-31")
        .await
        .unwrap();
    assert_eq!(secondary.len(), 1);
    assert_eq!(secondary[0].source, "apple_health");
}

#[tokio::test]
async fn a_cancelled_import_writes_nothing_even_after_some_batches_were_written() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let mut body = String::new();
    for i in 0..20_000 {
        body += &record(
            "BodyMass",
            "kg",
            &format!("{:.1}", 60.0 + (i % 400) as f64 / 10.0),
            "Balance",
            &crate::support::apple_export::stamp(i),
        );
    }
    let path = small_export(dir.path(), &body);
    let cancel = Arc::new(AtomicBool::new(true));

    let report = import_apple_health(&pool, path.to_str().unwrap(), cancel, &|_| {})
        .await
        .expect("import annule");

    assert!(report.cancelled);
    assert_eq!(report.tally.written(), 0);
    assert_eq!(count(&pool, "body_measures_legacy").await, 0);
}

#[tokio::test]
async fn an_unreadable_or_foreign_file_is_a_clear_error_and_writes_nothing() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let text = dir.path().join("notes.txt");
    std::fs::write(&text, "bonjour").unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let err = import_apple_health(&pool, text.to_str().unwrap(), cancel.clone(), &|_| {})
        .await
        .unwrap_err();
    assert!(err.to_string().contains(".zip"), "{err}");

    let broken = dir.path().join("casse.xml");
    std::fs::write(
        &broken,
        "<HealthData><Record type=\"HKQuantityTypeIdentifierBodyMass\" =x/>",
    )
    .unwrap();
    let err = import_apple_health(&pool, broken.to_str().unwrap(), cancel, &|_| {})
        .await
        .unwrap_err();
    assert!(err.to_string().contains("illisible"), "{err}");
    assert_eq!(counts(&pool).await, vec![0; 6]);
    let logged: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sync_log WHERE source = 'apple_health' AND status = 'error'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(logged, 1);
}

fn parse_all(path: &Path) -> (u64, u64) {
    let cancel = AtomicBool::new(false);
    with_export(path, Arc::new(AtomicU64::new(0)), |reader, _| {
        let mut mapper = Mapper::default();
        let mut buffer = Vec::new();
        let (mut outs, mut flushed) = (0u64, 0u64);
        parse_export(reader, wants, &cancel, |item| {
            mapper.feed(item, &mut buffer);
            if buffer.len() >= BATCH {
                outs += buffer.len() as u64;
                buffer.clear();
            }
            Ok(())
        })?;
        mapper.finish(&mut buffer);
        flushed += buffer.len() as u64;
        Ok((outs + flushed, flushed))
    })
    .unwrap()
}

#[test]
fn a_200000_record_export_is_read_in_a_few_megabytes_whatever_its_size() {
    let dir = tempfile::tempdir().unwrap();
    let xml = write_xml(dir.path(), 200_000);
    let archive = zip_of(dir.path(), &xml);
    let xml_bytes = std::fs::metadata(&xml).unwrap().len();
    assert!(xml_bytes > 40 * 1024 * 1024, "{xml_bytes}");

    let ((outs, _), peak_zip) = peak_of(|| parse_all(&archive));
    let (_, peak_xml) = peak_of(|| parse_all(&xml));
    eprintln!("pic zip {peak_zip} octets, pic xml {peak_xml} octets, sorties {outs}");

    assert!(outs >= 40_000, "{outs}");
    assert!(peak_zip < PEAK_CEILING_BYTES, "{peak_zip}");
    assert!(peak_xml < PEAK_CEILING_BYTES, "{peak_xml}");

    let (_, naive) = peak_of(|| std::fs::read_to_string(&xml).unwrap().len());
    assert!(
        naive > 10 * PEAK_CEILING_BYTES,
        "l'instrument ne voit pas un chargement entier : {naive}"
    );
}

#[tokio::test]
async fn a_200000_record_export_is_imported_with_every_expected_row() {
    let _gate = import_gate::importing().await;
    let dir = tempfile::tempdir().unwrap();
    let xml = write_xml(dir.path(), 200_000);
    let archive = zip_of(dir.path(), &xml);
    let pool = synk_lib::db::init_pool(dir.path().join("synk.db"))
        .await
        .unwrap();

    let report = import(&pool, &archive).await;

    assert_eq!(report.tally.observations, 40_000);
    assert!(report.tally.activity > 100, "{:?}", report.tally);
    assert_eq!(report.skipped.len(), 0, "{:?}", report.skipped);
    let steps_days: i64 = count(&pool, "activity").await;
    assert_eq!(steps_days as u64, report.tally.activity);
    let again = import(&pool, &archive).await;
    assert_eq!(again.tally.observations, 0);
    assert_eq!(again.tally.observations_ignored, 40_000);
    assert_eq!(count(&pool, "activity").await, steps_days);
}

#[tokio::test]
async fn a_three_megabyte_export_imports_completely_from_its_archive() {
    let _gate = import_gate::importing().await;
    let dir = tempfile::tempdir().unwrap();
    let xml = write_xml(dir.path(), 14_000);
    let size = std::fs::metadata(&xml).unwrap().len();
    assert!((3_000_000..4_000_000).contains(&size), "{size}");
    let archive = zip_of(dir.path(), &xml);
    let pool = memory_pool().await;

    let report = import(&pool, &archive).await;

    assert_eq!(report.tally.observations, 2_800);
    assert_eq!(count(&pool, "vitals_legacy").await, 1_400);
    assert_eq!(count(&pool, "body_measures_legacy").await, 1_400);
    assert!(report.skipped.is_empty(), "{:?}", report.skipped);
}

#[tokio::test]
async fn the_import_command_only_accepts_a_file_source_and_one_import_at_a_time() {
    let _gate = import_gate::importing().await;
    use synk_lib::commands::import::{import_file_inner, ImportControl};
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let path = small_export(dir.path(), &day_body());
    let control = ImportControl::default();
    let path = path.to_str().unwrap();

    let err = import_file_inner(&control, &pool, "oura", path, &|_| {})
        .await
        .unwrap_err();
    assert!(err.to_string().contains("oura"), "{err}");
    let err = import_file_inner(&control, &pool, "inconnue", path, &|_| {})
        .await
        .unwrap_err();
    assert!(err.to_string().contains("inconnue"), "{err}");

    let report = import_file_inner(&control, &pool, "apple_health", path, &|_| {})
        .await
        .expect("import");
    assert!(report.tally.written() > 0);
    assert!(
        !control.cancel(),
        "le verrou doit etre rendu apres l'import"
    );
}

#[tokio::test]
async fn the_source_list_describes_the_file_source_with_its_extensions() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let vault = crate::common::test_vault();
    let list = synk_lib::services::sources::list(&pool, vault.as_ref())
        .await
        .unwrap();
    let apple = list
        .iter()
        .find(|s| s.id == "apple_health")
        .expect("source");
    assert_eq!(apple.kind, "file");
    assert_eq!(apple.extensions, vec!["zip", "xml"]);
    assert!(!apple.connected);
    let oura = list.iter().find(|s| s.id == "oura").unwrap();
    assert!(oura.extensions.is_empty());
}

#[test]
#[ignore = "ecrit un export synthetique pour la verification a l'ecran : SYNK_SAMPLE_EXPORT=<dossier>"]
fn write_a_sample_export_for_the_screen_check() {
    let Ok(target) = std::env::var("SYNK_SAMPLE_EXPORT") else {
        return;
    };
    let dir = std::path::PathBuf::from(target);
    std::fs::create_dir_all(&dir).unwrap();
    let mut body = String::new();
    for d in 0..45 {
        let day = chrono::NaiveDate::from_ymd_opt(2026, 9, 10).unwrap() + chrono::Duration::days(d);
        let at = |h: u32| format!("{} {h:02}:15:00 +0200", day.format("%Y-%m-%d"));
        body += &record(
            "BodyMass",
            "lb",
            &format!("{:.1}", 165.0 - d as f64 * 0.1),
            "Withings",
            &at(7),
        );
        body += &record(
            "StepCount",
            "count",
            &format!("{}", 5000 + d * 70),
            "iPhone",
            &at(10),
        );
        body += &record(
            "StepCount",
            "count",
            &format!("{}", 6500 + d * 60),
            "Apple Watch",
            &at(15),
        );
        body += &record(
            "ActiveEnergyBurned",
            "kcal",
            &format!("{}", 350 + d),
            "Apple Watch",
            &at(15),
        );
        body += &record(
            "BloodPressureSystolic",
            "mmHg",
            &format!("{}", 118 + d % 9),
            "Tensio",
            &at(8),
        );
        body += &record(
            "BloodPressureDiastolic",
            "mmHg",
            &format!("{}", 76 + d % 5),
            "Tensio",
            &at(8),
        );
        body += &format!(
            " <Record type=\"HKCategoryTypeIdentifierSleepAnalysis\" sourceName=\"Apple Watch\" startDate=\"{} 23:10:00 +0200\" endDate=\"{} 06:40:00 +0200\" value=\"HKCategoryValueSleepAnalysisAsleepCore\"/>\n",
            day.format("%Y-%m-%d"),
            (day + chrono::Duration::days(1)).format("%Y-%m-%d")
        );
    }
    let xml = crate::support::apple_export::small_export(&dir, &body);
    std::fs::rename(xml, dir.join("export-synthetique.zip")).unwrap();
}

#[tokio::test]
async fn a_day_total_that_changed_between_two_exports_is_updated_and_counted() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let dir = tempfile::tempdir().unwrap();
    let morning = "2026-10-25 07:30:00 +0200";
    let first = small_export(
        dir.path(),
        &record("StepCount", "count", "4000", "iPhone", morning),
    );
    import(&pool, &first).await;
    let dir2 = tempfile::tempdir().unwrap();
    let body = record("StepCount", "count", "4000", "iPhone", morning)
        + &record(
            "StepCount",
            "count",
            "1500",
            "iPhone",
            "2026-10-25 20:00:00 +0200",
        );
    let second = small_export(dir2.path(), &body);

    let report = import(&pool, &second).await;

    assert_eq!(report.tally.activity, 1);
    let steps: i64 = sqlx::query_scalar("SELECT steps FROM activity WHERE source = 'apple_health'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(steps, 5500);
    assert_eq!(count(&pool, "activity").await, 1);
}
