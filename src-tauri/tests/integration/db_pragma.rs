use crate::support::import_gate;
use synk_lib::db::init_pool;

#[tokio::test]
async fn init_pool_enables_foreign_keys() {
    let dir = crate::support::temp_dirs::tempdir().unwrap();
    let tmp = dir.path().join("synk_test_fk.db");

    let pool = init_pool(tmp.clone()).await.unwrap();
    let (fk,): (i64,) = sqlx::query_as("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(fk, 1);

    pool.close().await;
}

#[tokio::test]
async fn init_pool_sets_synchronous_normal() {
    let dir = crate::support::temp_dirs::tempdir().unwrap();
    let tmp = dir.path().join("synk_test_sync.db");

    let pool = init_pool(tmp.clone()).await.unwrap();
    let (sync,): (i64,) = sqlx::query_as("PRAGMA synchronous")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(sync, 1, "synchronous should be NORMAL (=1)");

    pool.close().await;
}

#[tokio::test]
async fn zzz_probe_fk_after_migration009() {
    let dir = crate::support::temp_dirs::tempdir().unwrap();
    let tmp = dir.path().join("synk_probe_fk.db");

    let pool = init_pool(tmp.clone()).await.unwrap();

    // Sonde toutes les connexions du pool (max 5) : la migration 009 a tourne
    // sur UNE connexion avec PRAGMA foreign_keys=OFF puis =ON. Si une connexion
    // revient au pool avec FK=OFF, on le detecte ici.
    let mut conns = Vec::new();
    for _ in 0..5 {
        let mut c = pool.acquire().await.unwrap();
        let (fk,): (i64,) = sqlx::query_as("PRAGMA foreign_keys")
            .fetch_one(&mut *c)
            .await
            .unwrap();
        conns.push((fk, c));
    }
    for (i, (fk, _)) in conns.iter().enumerate() {
        println!("PROBE conn#{i} foreign_keys={fk}");
    }
    let any_off = conns.iter().any(|(fk, _)| *fk == 0);
    println!("PROBE any_off={any_off}");
    assert!(
        !any_off,
        "une connexion du pool a foreign_keys=OFF apres migration 009"
    );

    drop(conns);
    pool.close().await;
}

#[tokio::test]
async fn init_pool_sets_busy_timeout() {
    let _gate = import_gate::no_import_running().await;
    let dir = crate::support::temp_dirs::tempdir().unwrap();
    let tmp = dir.path().join("synk_test_timeout.db");

    let pool = init_pool(tmp.clone()).await.unwrap();
    let (t,): (i64,) = sqlx::query_as("PRAGMA busy_timeout")
        .fetch_one(&pool)
        .await
        .unwrap();
    let expected = i64::from(synk_lib::db::gate::busy_timeout_ms());
    assert!(t == 5000 || t == expected, "{t}");

    pool.close().await;
}
