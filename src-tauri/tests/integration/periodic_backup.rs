use sqlx::SqlitePool;
use std::time::{Duration, SystemTime};
use synk_lib::boot::{periodic_backup_inner, PeriodicOutcome, PERIODIC_KEEP};

async fn bootstrapped_pool(dir: &tempfile::TempDir) -> SqlitePool {
    let pool = synk_lib::db::init_pool(dir.path().join("synk.db"))
        .await
        .expect("base temporaire initialisee");
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, source, weight) \
         VALUES ('2026-09-01T08:00:00Z', 'manual', 72.5)",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

fn daily_count(dir: &tempfile::TempDir) -> usize {
    std::fs::read_dir(dir.path().join("backups").join("daily"))
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|x| x == "db")
        })
        .count()
}

#[tokio::test]
async fn writes_a_readable_first_backup() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    let outcome = periodic_backup_inner(&pool, dir.path(), SystemTime::now())
        .await
        .unwrap();
    let PeriodicOutcome::Written(path) = outcome else {
        panic!("premiere sauvegarde attendue, obtenu {outcome:?}");
    };
    let copy = SqlitePool::connect(&format!("sqlite:{}?mode=ro", path.display()))
        .await
        .unwrap();
    let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM body_measures_legacy")
        .fetch_one(&copy)
        .await
        .unwrap();
    assert_eq!(n, 1);
}

#[tokio::test]
async fn skips_when_the_latest_backup_is_fresh() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    let now = SystemTime::now();
    periodic_backup_inner(&pool, dir.path(), now).await.unwrap();
    let second = periodic_backup_inner(&pool, dir.path(), now + Duration::from_secs(3600))
        .await
        .unwrap();
    assert_eq!(second, PeriodicOutcome::Fresh);
    assert_eq!(daily_count(&dir), 1);
}

#[tokio::test]
async fn keeps_only_the_fourteen_latest() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    let start = SystemTime::now();
    for day in 0..(PERIODIC_KEEP as u64 + 2) {
        let now = start + Duration::from_secs(day * 21 * 3600);
        let outcome = periodic_backup_inner(&pool, dir.path(), now).await.unwrap();
        assert!(
            matches!(outcome, PeriodicOutcome::Written(_)),
            "jour {day} : {outcome:?}"
        );
    }
    assert_eq!(daily_count(&dir), PERIODIC_KEEP);
}

#[tokio::test]
async fn failure_is_reported_not_swallowed() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    std::fs::create_dir_all(dir.path().join("backups")).unwrap();
    std::fs::write(dir.path().join("backups").join("daily"), b"pas un dossier").unwrap();
    let err = periodic_backup_inner(&pool, dir.path(), SystemTime::now())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("daily"), "obtenu : {err}");
}

#[test]
fn a_recorded_failure_stays_readable_for_a_late_listener() {
    let failure = synk_lib::boot::BackupFailure::default();
    assert!(failure.current().is_none());
    failure.record("disque plein");
    let first = failure.current().unwrap();
    assert_eq!(first.kind(), "Internal");
    assert_eq!(first.detail(), "disque plein");
    assert_eq!(failure.current().unwrap().detail(), "disque plein");
}

#[test]
fn an_hourly_retry_reports_the_same_failure_once_and_a_success_clears_it() {
    let failure = synk_lib::boot::BackupFailure::default();
    assert!(failure.record("disque plein"));
    assert!(!failure.record("disque plein"));
    assert!(failure.record("acces refuse"));
    failure.clear();
    assert!(failure.current().is_none());
    assert!(failure.record("acces refuse"));
}

async fn one_round(
    pool: &SqlitePool,
    dir: &std::path::Path,
    now: SystemTime,
) -> Option<std::path::PathBuf> {
    match periodic_backup_inner(pool, dir, now).await.unwrap() {
        PeriodicOutcome::Written(path) => {
            std::fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(now)
                .unwrap();
            Some(path)
        }
        PeriodicOutcome::Fresh => None,
    }
}

#[tokio::test]
async fn a_long_session_writes_a_backup_every_day_without_a_restart() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    let start = SystemTime::now();
    let hours = Arc::new(AtomicU32::new(0));
    let written = Arc::new(AtomicU32::new(0));
    let finished = Arc::new(tokio::sync::Notify::new());

    let tick = {
        let (hours, written, finished) = (hours.clone(), written.clone(), finished.clone());
        let (pool, path) = (pool.clone(), dir.path().to_path_buf());
        move || {
            let (hours, written, finished) = (hours.clone(), written.clone(), finished.clone());
            let (pool, path) = (pool.clone(), path.clone());
            async move {
                let elapsed = hours.fetch_add(1, Ordering::SeqCst);
                let now = start + Duration::from_secs(u64::from(elapsed) * 3600);
                if one_round(&pool, &path, now).await.is_some() {
                    written.fetch_add(1, Ordering::SeqCst);
                }
                if elapsed == 50 {
                    finished.notify_one();
                }
            }
        }
    };
    tokio::select! {
        () = synk_lib::boot::run_every(Duration::from_millis(2), tick) => unreachable!(),
        () = finished.notified() => {}
    }
    assert!(hours.load(Ordering::SeqCst) >= 51);
    assert_eq!(written.load(Ordering::SeqCst), 3);
    assert_eq!(daily_count(&dir), 3);
}

#[test]
fn the_backup_failed_event_is_a_typed_redacted_error_like_the_other_events() {
    let payload = synk_lib::boot::backup_failure_payload(
        "copie vers https://exemple.test/x?access_token=abc123&a=1 impossible",
    );
    assert_eq!(payload["kind"], "Internal");
    let detail = payload["detail"].as_str().unwrap();
    assert!(detail.contains("copie vers"), "{detail}");
    assert!(!detail.contains("abc123"), "{detail}");
    assert!(payload["message"].as_str().unwrap().contains("copie vers"));
}

#[test]
fn the_state_read_at_mount_is_redacted_too() {
    let failure = synk_lib::boot::BackupFailure::default();
    failure.record("echec access_token=abc123");
    let seen = serde_json::to_value(failure.current().unwrap()).unwrap();
    assert_eq!(seen["kind"], "Internal");
    assert!(!seen["detail"].as_str().unwrap().contains("abc123"));
}
