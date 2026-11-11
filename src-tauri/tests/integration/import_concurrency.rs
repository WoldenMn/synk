use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::support::import_gate;
use synk_lib::commands::journal::{upsert_journal_entry_inner, JournalInput};
use synk_lib::sources::import::pipeline::{run_import, send_all, Produced};
use synk_lib::sources::import::rows::{Out, Skipped, WorkoutRow};
use synk_lib::sources::SourceId;

fn workout(i: usize) -> Out {
    Out::Workout(WorkoutRow {
        date: "2026-10-01".into(),
        external_id: format!("concurrence-{i}"),
        start_at: format!("2026-10-01T10:{:02}:00Z", i % 60),
        end_at: format!("2026-10-01T11:{:02}:00Z", i % 60),
        category_label: "Course".into(),
        duration: 3600,
        ..WorkoutRow::default()
    })
}

fn entry(content: &str) -> JournalInput {
    JournalInput {
        date: "2026-10-02".into(),
        content: content.into(),
        tags: None,
    }
}

async fn held_import(
    dir: &std::path::Path,
    cancel_at_end: bool,
) -> (
    sqlx::SqlitePool,
    tokio::task::JoinHandle<
        Result<synk_lib::sources::import::pipeline::ImportReport, synk_lib::error::AppError>,
    >,
    mpsc::Sender<()>,
) {
    let pool = synk_lib::db::init_pool(dir.join("synk.db")).await.unwrap();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let started = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&started);
    let cancel = Arc::new(AtomicBool::new(false));
    let import = {
        let pool = pool.clone();
        let cancel = Arc::clone(&cancel);
        tokio::spawn(async move {
            run_import(
                &pool,
                SourceId::TRACE,
                Arc::clone(&cancel),
                &|_| {},
                move |tx, flag_cancel, _| {
                    send_all(tx, (0..50).map(workout).collect())?;
                    flag.store(true, Ordering::SeqCst);
                    release_rx.recv().ok();
                    if cancel_at_end {
                        flag_cancel.store(true, Ordering::SeqCst);
                    }
                    Ok(Produced {
                        cancelled: false,
                        skipped: Skipped::default(),
                    })
                },
            )
            .await
        })
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    while !started.load(Ordering::SeqCst) {
        assert!(Instant::now() < deadline, "le producteur n'a pas demarre");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
    (pool, import, release_tx)
}

async fn workouts(pool: &sqlx::SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM workouts WHERE source = 'trace'")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_write_during_an_import_is_refused_at_once_and_clearly_and_the_import_still_commits() {
    let _gate = import_gate::importing().await;
    let dir = tempfile::tempdir().unwrap();
    let (pool, import, release) = held_import(dir.path(), false).await;

    let clock = Instant::now();
    let err = upsert_journal_entry_inner(&pool, entry("pendant"))
        .await
        .unwrap_err();
    let waited = clock.elapsed();
    assert!(
        waited < Duration::from_millis(2500),
        "attente de {waited:?}"
    );
    assert!(err.to_string().contains("import"), "{err}");
    assert!(synk_lib::db::gate::ensure_idle().is_err());

    release.send(()).unwrap();
    let report = import.await.unwrap().unwrap();
    assert!(!report.cancelled);
    assert_eq!(workouts(&pool).await, 50);

    upsert_journal_entry_inner(&pool, entry("apres"))
        .await
        .expect("l'ecriture reprend a la fin de l'import");
}

#[tokio::test(flavor = "multi_thread")]
async fn cancelling_a_held_import_writes_nothing_and_frees_the_database() {
    let _gate = import_gate::importing().await;
    let dir = tempfile::tempdir().unwrap();
    let (pool, import, release) = held_import(dir.path(), true).await;
    release.send(()).unwrap();
    let report = import.await.unwrap().unwrap();
    assert!(report.cancelled);
    assert_eq!(workouts(&pool).await, 0);
    upsert_journal_entry_inner(&pool, entry("libre"))
        .await
        .expect("la base est libre apres l'annulation");
}
