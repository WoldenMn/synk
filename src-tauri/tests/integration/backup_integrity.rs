use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use synk_lib::commands::export::restore_latest_backup_inner;
use synk_lib::services::backup::{copy_database_safely, verify_backup};

fn wal_len(db_path: &std::path::Path) -> u64 {
    std::fs::metadata(db_path.with_extension("db-wal"))
        .map(|m| m.len())
        .unwrap_or(0)
}

// Reproduit une session terminee sans checkpoint : l'ecriture vit encore dans le
// fichier -wal et est absente du fichier principal. Le pool reste ouvert, et
// c'est necessaire : mesure faite, fermer la derniere connexion checkpointe et
// vide le -wal, il ne resterait alors plus rien a prouver.
async fn pool_with_a_write_left_in_the_wal(db_path: &std::path::Path) -> SqlitePool {
    let url = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("PRAGMA wal_autocheckpoint=0")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE mesures (id INTEGER PRIMARY KEY, valeur REAL)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO mesures (valeur) VALUES (82.0)")
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        wal_len(db_path) > 0,
        "le montage doit laisser l'ecriture dans le -wal, sinon le test ne prouve rien"
    );
    pool
}

// Ouvre la sauvegarde comme la restauration la lira. `restore_latest_backup_inner`
// recopie uniquement le fichier `.db`, jamais les `-wal` / `-shm` poses a cote :
// une donnee restee dans le `-wal` d'une sauvegarde est perdue a la restauration,
// en silence. On juge donc la sauvegarde sur son seul `.db`, isole.
async fn open_as_restore_would(backup_path: &std::path::Path) -> SqlitePool {
    let solo_dir = backup_path.parent().unwrap().join("restaure");
    std::fs::create_dir_all(&solo_dir).unwrap();
    let solo = solo_dir.join("synk.db");
    std::fs::copy(backup_path, &solo).unwrap();
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite:{}", solo.display()))
        .await
        .unwrap()
}

#[tokio::test]
async fn a_backup_contains_writes_still_sitting_in_the_wal() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("synk.db");
    let pool = pool_with_a_write_left_in_the_wal(&db_path).await;

    let backup_path = dir.path().join("backup.db");
    copy_database_safely(&db_path, &backup_path, Some(&pool))
        .await
        .unwrap();
    pool.close().await;

    let restored = open_as_restore_would(&backup_path).await;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mesures")
        .fetch_one(&restored)
        .await
        .unwrap();
    assert_eq!(
        count, 1,
        "la sauvegarde doit contenir l'ecriture restee dans le WAL"
    );
}

#[tokio::test]
async fn a_backup_without_a_pool_opens_its_own_connection() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("synk.db");
    let pool = pool_with_a_write_left_in_the_wal(&db_path).await;

    let backup_path = dir.path().join("backup.db");
    copy_database_safely(&db_path, &backup_path, None)
        .await
        .unwrap();
    pool.close().await;

    let restored = open_as_restore_would(&backup_path).await;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mesures")
        .fetch_one(&restored)
        .await
        .unwrap();
    assert_eq!(
        count, 1,
        "la sauvegarde doit contenir l'ecriture restee dans le WAL"
    );
}

async fn integrity_report(pool: &SqlitePool) -> Vec<String> {
    sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_all(pool)
        .await
        .unwrap()
}

fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn a_backup_keeps_writes_that_an_open_reader_holds_out_of_the_main_file() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("synk.db");
    let writer = pool_with_a_write_left_in_the_wal(&db_path).await;

    let reader = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&db_path))
        .await
        .unwrap();
    let mut snapshot = reader.begin().await.unwrap();
    let seen: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mesures")
        .fetch_one(&mut *snapshot)
        .await
        .unwrap();
    assert_eq!(seen, 1);

    sqlx::query("INSERT INTO mesures (valeur) VALUES (81.5)")
        .execute(&writer)
        .await
        .unwrap();

    let backup_path = dir.path().join("backup.db");
    let outcome = copy_database_safely(&db_path, &backup_path, Some(&writer)).await;
    snapshot.rollback().await.unwrap();
    reader.close().await;
    writer.close().await;
    outcome.unwrap();

    let restored = open_as_restore_would(&backup_path).await;
    let valeurs: Vec<f64> = sqlx::query_scalar("SELECT valeur FROM mesures ORDER BY id")
        .fetch_all(&restored)
        .await
        .unwrap();
    assert_eq!(
        valeurs,
        vec![82.0, 81.5],
        "une ecriture validee pendant qu'un lecteur tient un instantane doit figurer dans le .db de la sauvegarde"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_backup_taken_during_concurrent_writes_is_a_consistent_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("synk.db");
    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&db_path)
                .create_if_missing(true)
                .pragma("wal_autocheckpoint", "1"),
        )
        .await
        .unwrap();
    for statement in [
        "PRAGMA journal_mode=WAL",
        "CREATE TABLE lots (id INTEGER PRIMARY KEY, lot INTEGER NOT NULL, charge TEXT NOT NULL)",
    ] {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }

    let stop = Arc::new(AtomicBool::new(false));
    let writer = tokio::spawn({
        let pool = pool.clone();
        let stop = Arc::clone(&stop);
        async move {
            let charge = "x".repeat(2000);
            let mut lot: i64 = 0;
            while !stop.load(Ordering::Relaxed) {
                let mut tx = pool.begin().await.unwrap();
                for _ in 0..2 {
                    sqlx::query("INSERT INTO lots (lot, charge) VALUES (?, ?)")
                        .bind(lot)
                        .bind(&charge)
                        .execute(&mut *tx)
                        .await
                        .unwrap();
                }
                tx.commit().await.unwrap();
                lot += 1;
            }
            lot
        }
    });

    let committed = || async {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT lot) FROM lots")
            .fetch_one(&pool)
            .await
            .unwrap()
    };
    while committed().await < 300 {
        assert!(
            !writer.is_finished(),
            "l'ecrivain s'est arrete avant les sauvegardes"
        );
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    let before = committed().await;

    let mut backups = Vec::new();
    for i in 0..6 {
        let backup_path = dir.path().join(format!("backup_{i}.db"));
        copy_database_safely(&db_path, &backup_path, Some(&pool))
            .await
            .unwrap();
        backups.push(backup_path);
    }

    let after = committed().await;
    stop.store(true, Ordering::Relaxed);
    let lots_written = writer.await.unwrap();
    pool.close().await;
    assert!(
        after > before,
        "l'ecrivain doit avoir progresse pendant les sauvegardes ({before} -> {after}), sinon le test ne prouve rien"
    );

    for backup_path in &backups {
        let restored = open_as_restore_would(backup_path).await;
        assert_eq!(integrity_report(&restored).await, vec!["ok".to_string()]);
        let torn: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM (SELECT lot FROM lots GROUP BY lot HAVING COUNT(*) != 2)",
        )
        .fetch_one(&restored)
        .await
        .unwrap();
        assert_eq!(torn, 0, "aucune transaction ne doit etre coupee en deux");
        let lots: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT lot) FROM lots")
            .fetch_one(&restored)
            .await
            .unwrap();
        assert!(
            lots >= before && lots <= lots_written,
            "{lots} lots sauvegardes, attendu entre {before} et {lots_written}"
        );
        restored.close().await;
    }
}

async fn second_connection(db_path: &Path) -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(db_path))
        .await
        .unwrap()
}

#[tokio::test]
async fn a_backup_taken_while_a_write_is_pending_restores_every_committed_row() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("synk.db");
    let app_pool = pool_with_a_write_left_in_the_wal(&db_path).await;
    let reader = second_connection(&db_path).await;
    let writer = second_connection(&db_path).await;

    let mut old_snapshot = reader.begin().await.unwrap();
    let seen: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mesures")
        .fetch_one(&mut *old_snapshot)
        .await
        .unwrap();
    assert_eq!(seen, 1);
    sqlx::query("INSERT INTO mesures (valeur) VALUES (81.5)")
        .execute(&app_pool)
        .await
        .unwrap();
    let mut pending_write = writer.begin().await.unwrap();
    sqlx::query("INSERT INTO mesures (valeur) VALUES (70.0)")
        .execute(&mut *pending_write)
        .await
        .unwrap();

    let auto_dir = dir.path().join("backups").join("auto");
    std::fs::create_dir_all(&auto_dir).unwrap();
    let backup_path = auto_dir.join("synk_2026-10-16_10-00-00_pre-v16.db");
    let outcome = copy_database_safely(&db_path, &backup_path, Some(&app_pool)).await;
    pending_write.commit().await.unwrap();
    old_snapshot.rollback().await.unwrap();
    for pool in [writer, reader, app_pool] {
        pool.close().await;
    }
    outcome.unwrap();

    let report = restore_latest_backup_inner(dir.path()).await.unwrap();
    assert!(
        report.skipped.is_empty(),
        "la sauvegarde ne doit pas etre ecartee : {:?}",
        report.skipped
    );
    assert_eq!(Path::new(&report.restored_from), backup_path.as_path());
    let restored = second_connection(&db_path).await;
    assert_eq!(integrity_report(&restored).await, vec!["ok".to_string()]);
    let valeurs: Vec<f64> = sqlx::query_scalar("SELECT valeur FROM mesures ORDER BY id")
        .fetch_all(&restored)
        .await
        .unwrap();
    restored.close().await;
    assert_eq!(
        valeurs,
        vec![82.0, 81.5],
        "la restauration doit rendre chaque ecriture validee avant la sauvegarde, et l'ecriture alors en cours ne doit pas y figurer"
    );
}

async fn damaged_database(db_path: &Path) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(true),
        )
        .await
        .unwrap();
    sqlx::query("CREATE TABLE mesures (id INTEGER PRIMARY KEY, note TEXT NOT NULL)")
        .execute(&pool)
        .await
        .unwrap();
    for i in 0..200 {
        sqlx::query("INSERT INTO mesures (note) VALUES (?)")
            .bind(format!("{i:0>500}"))
            .execute(&pool)
            .await
            .unwrap();
    }
    pool.close().await;

    let mut bytes = std::fs::read(db_path).unwrap();
    assert!(
        bytes.len() > 4096 * 8,
        "la base doit couvrir plusieurs pages"
    );
    bytes[4096 * 2..4096 * 4].fill(0xA5);
    std::fs::write(db_path, bytes).unwrap();

    let probe = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(db_path)
                .read_only(true),
        )
        .await
        .unwrap();
    let report: Result<Vec<String>, _> = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_all(&probe)
        .await;
    probe.close().await;
    assert!(
        !matches!(report.as_deref(), Ok([ok]) if ok == "ok"),
        "le montage doit produire une base reellement endommagee, sinon le test ne prouve rien"
    );
}

#[tokio::test]
async fn a_damaged_database_is_refused_without_leaving_a_backup_file() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("synk.db");
    damaged_database(&db_path).await;
    let backups_dir = dir.path().join("backups");
    std::fs::create_dir_all(&backups_dir).unwrap();

    let backup_path = backups_dir.join("synk_2026-09-28_10-00-00.db");
    let outcome = copy_database_safely(&db_path, &backup_path, None).await;

    let message =
        outcome.expect_err("une base endommagee ne doit pas donner une sauvegarde valide");
    assert!(
        message.detail().contains("Sauvegarde"),
        "le message doit dire ce qui a echoue : {message}"
    );
    assert!(
        files_in(&backups_dir).is_empty(),
        "aucun fichier ne doit rester : {:?}",
        files_in(&backups_dir)
    );
}

#[tokio::test]
async fn an_existing_file_is_never_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("synk.db");
    let pool = pool_with_a_write_left_in_the_wal(&db_path).await;

    let backup_path = dir.path().join("backup.db");
    std::fs::write(&backup_path, b"SAUVEGARDE_PRECEDENTE").unwrap();
    let outcome = copy_database_safely(&db_path, &backup_path, Some(&pool)).await;
    pool.close().await;

    let message = outcome.expect_err("un fichier deja present ne doit pas etre ecrase");
    assert!(
        message.detail().contains("existe déjà"),
        "message obtenu : {message}"
    );
    assert_eq!(
        std::fs::read(&backup_path).unwrap(),
        b"SAUVEGARDE_PRECEDENTE"
    );
}

#[tokio::test]
async fn a_backup_restores_every_row_it_holds() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("synk.db");
    let pool = pool_with_a_write_left_in_the_wal(&db_path).await;
    let auto_dir = dir.path().join("backups").join("auto");
    std::fs::create_dir_all(&auto_dir).unwrap();

    copy_database_safely(
        &db_path,
        &auto_dir.join("synk_2026-09-28_10-00-00_pre-v15.db"),
        Some(&pool),
    )
    .await
    .unwrap();
    sqlx::query("INSERT INTO mesures (valeur) VALUES (70.0)")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;

    let report = restore_latest_backup_inner(dir.path()).await.unwrap();
    assert!(Path::new(&report.broken_kept_at).exists());

    let restored = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&db_path))
        .await
        .unwrap();
    assert_eq!(integrity_report(&restored).await, vec!["ok".to_string()]);
    let valeurs: Vec<f64> = sqlx::query_scalar("SELECT valeur FROM mesures ORDER BY id")
        .fetch_all(&restored)
        .await
        .unwrap();
    restored.close().await;
    assert_eq!(valeurs, vec![82.0]);
}

#[tokio::test]
async fn verification_accepts_a_sound_copy() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("synk.db");
    let pool = pool_with_a_write_left_in_the_wal(&db_path).await;
    let backup_path = dir.path().join("backup.db");
    copy_database_safely(&db_path, &backup_path, Some(&pool))
        .await
        .unwrap();
    pool.close().await;

    verify_backup(&backup_path).await.unwrap();
    let around_backup: Vec<String> = files_in(dir.path())
        .into_iter()
        .filter(|name| name.starts_with("backup"))
        .collect();
    assert_eq!(
        around_backup,
        vec!["backup.db".to_string()],
        "ni -wal, ni -shm, ni fichier temporaire ne doivent rester a cote de la sauvegarde"
    );
}

#[tokio::test]
async fn verification_rejects_a_damaged_copy() {
    let dir = tempfile::tempdir().unwrap();
    let damaged = dir.path().join("abimee.db");
    damaged_database(&damaged).await;

    let message = verify_backup(&damaged)
        .await
        .expect_err("une copie endommagee doit etre refusee");
    assert!(
        message.detail().contains("intégrité"),
        "message obtenu : {message}"
    );
}
