use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::Connection;
use std::path::{Path, PathBuf};
use synk_lib::commands::export::restore_latest_backup_inner;

// Helpers

fn make_auto_dir(root: &Path) -> PathBuf {
    let dir = root.join("backups").join("auto");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// Une vraie base SQLite, en journal rollback comme celles que produit VACUUM INTO,
// marquee pour qu'un test sache laquelle a ete restauree.
async fn write_backup(dir: &Path, name: &str, marker: &str) -> PathBuf {
    let path = dir.join(name);
    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);
    let mut conn = SqliteConnection::connect_with(&options).await.unwrap();
    sqlx::query("CREATE TABLE marqueur (texte TEXT NOT NULL)")
        .execute(&mut conn)
        .await
        .unwrap();
    sqlx::query("INSERT INTO marqueur (texte) VALUES (?)")
        .bind(marker)
        .execute(&mut conn)
        .await
        .unwrap();
    conn.close().await.unwrap();
    path
}

fn write_garbage(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, b"PAS UNE BASE SQLITE").unwrap();
    path
}

async fn read_marker(db_path: &Path) -> String {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .read_only(true);
    let mut conn = SqliteConnection::connect_with(&options).await.unwrap();
    let marker: String = sqlx::query_scalar("SELECT texte FROM marqueur")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    conn.close().await.unwrap();
    marker
}

fn set_mtime(path: &Path, when: std::time::SystemTime) {
    std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(when)
        .unwrap();
}

fn companion(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn broken_leftovers(root: &Path) -> Vec<String> {
    std::fs::read_dir(root)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains("broken") || name.contains("restoring"))
        .collect()
}

// --- Tests ---

#[tokio::test]
async fn restore_fails_when_no_auto_dir() {
    let tmp = tempfile::TempDir::new().unwrap();
    let result = restore_latest_backup_inner(tmp.path()).await;
    assert!(result
        .unwrap_err()
        .detail()
        .contains("Aucune sauvegarde disponible"));
}

#[tokio::test]
async fn restore_fails_when_auto_dir_empty() {
    let tmp = tempfile::TempDir::new().unwrap();
    make_auto_dir(tmp.path());
    let result = restore_latest_backup_inner(tmp.path()).await;
    assert!(result
        .unwrap_err()
        .detail()
        .contains("Aucune sauvegarde disponible"));
}

#[tokio::test]
async fn restore_copies_backup_to_db_path() {
    let tmp = tempfile::TempDir::new().unwrap();
    let auto_dir = make_auto_dir(tmp.path());
    write_backup(&auto_dir, "synk_2026-01-01_pre-v14.db", "SAUVEGARDE").await;

    let report = restore_latest_backup_inner(tmp.path()).await.unwrap();

    let db_path = tmp.path().join("synk.db");
    assert_eq!(read_marker(&db_path).await, "SAUVEGARDE");
    assert!(report.skipped.is_empty());
    assert!(
        !tmp.path().join("synk.db.restoring").exists(),
        "la copie temporaire doit devenir synk.db, pas rester a cote"
    );
}

#[tokio::test]
async fn restore_preserves_existing_db_as_broken() {
    let tmp = tempfile::TempDir::new().unwrap();
    let auto_dir = make_auto_dir(tmp.path());
    write_backup(&auto_dir, "synk_2026-01-01_pre-v14.db", "SAUVEGARDE").await;
    let db_path = tmp.path().join("synk.db");
    std::fs::write(&db_path, b"BROKEN_DB").unwrap();

    let report = restore_latest_backup_inner(tmp.path()).await.unwrap();

    assert_eq!(read_marker(&db_path).await, "SAUVEGARDE");
    let broken_path = Path::new(&report.broken_kept_at);
    assert_eq!(std::fs::read(broken_path).unwrap(), b"BROKEN_DB");
}

#[tokio::test]
async fn restore_picks_most_recent_backup_when_multiple() {
    let tmp = tempfile::TempDir::new().unwrap();
    let auto_dir = make_auto_dir(tmp.path());
    let old_path = write_backup(&auto_dir, "synk_2026-01-01_pre-v13.db", "ANCIENNE").await;
    let new_path = write_backup(&auto_dir, "synk_2026-06-27_pre-v14.db", "RECENTE").await;
    let now = std::time::SystemTime::now();
    set_mtime(&old_path, now - std::time::Duration::from_secs(3600));
    set_mtime(&new_path, now);

    restore_latest_backup_inner(tmp.path()).await.unwrap();

    assert_eq!(read_marker(&tmp.path().join("synk.db")).await, "RECENTE");
}

#[tokio::test]
async fn restore_skips_a_corrupt_latest_backup_and_uses_the_previous_valid_one() {
    let tmp = tempfile::TempDir::new().unwrap();
    let auto_dir = make_auto_dir(tmp.path());
    let valid = write_backup(&auto_dir, "synk_2026-01-01_pre-v13.db", "ANCIENNE").await;
    let corrupt = write_garbage(&auto_dir, "synk_2026-06-27_pre-v14.db");
    let now = std::time::SystemTime::now();
    set_mtime(&valid, now - std::time::Duration::from_secs(3600));
    set_mtime(&corrupt, now);

    let report = restore_latest_backup_inner(tmp.path()).await.unwrap();

    assert_eq!(read_marker(&tmp.path().join("synk.db")).await, "ANCIENNE");
    assert_eq!(report.skipped.len(), 1, "rapport : {:?}", report.skipped);
    assert!(report.skipped[0].contains("synk_2026-06-27_pre-v14.db"));
}

#[tokio::test]
async fn restore_refuses_when_no_backup_is_usable_and_leaves_the_current_db_alone() {
    let tmp = tempfile::TempDir::new().unwrap();
    let auto_dir = make_auto_dir(tmp.path());
    write_garbage(&auto_dir, "synk_2026-06-27_pre-v14.db");
    let db_path = tmp.path().join("synk.db");
    std::fs::write(&db_path, b"BASE_COURANTE").unwrap();
    std::fs::write(companion(&db_path, "-wal"), b"WAL_COURANT").unwrap();

    let message = restore_latest_backup_inner(tmp.path())
        .await
        .expect_err("une sauvegarde illisible ne doit jamais remplacer la base");

    assert!(
        message.detail().contains("Aucune sauvegarde utilisable"),
        "{message}"
    );
    assert_eq!(std::fs::read(&db_path).unwrap(), b"BASE_COURANTE");
    assert_eq!(
        std::fs::read(companion(&db_path, "-wal")).unwrap(),
        b"WAL_COURANT"
    );
    let leftovers = broken_leftovers(tmp.path());
    assert!(leftovers.is_empty(), "rien ne doit bouger : {leftovers:?}");
}

#[tokio::test]
async fn restore_moves_the_journal_of_the_current_db_aside_with_it() {
    let tmp = tempfile::TempDir::new().unwrap();
    let auto_dir = make_auto_dir(tmp.path());
    write_backup(&auto_dir, "synk_2026-01-01_pre-v14.db", "SAUVEGARDE").await;
    let db_path = tmp.path().join("synk.db");
    std::fs::write(&db_path, b"BROKEN_DB").unwrap();
    std::fs::write(companion(&db_path, "-wal"), b"BROKEN_WAL").unwrap();
    std::fs::write(companion(&db_path, "-shm"), b"BROKEN_SHM").unwrap();

    let report = restore_latest_backup_inner(tmp.path()).await.unwrap();

    for suffix in ["-wal", "-shm"] {
        assert!(
            !companion(&db_path, suffix).exists(),
            "le {suffix} de l'ancienne base serait rejoue sur la sauvegarde restauree"
        );
    }
    assert_eq!(read_marker(&db_path).await, "SAUVEGARDE");
    let broken_path = Path::new(&report.broken_kept_at);
    assert_eq!(std::fs::read(broken_path).unwrap(), b"BROKEN_DB");
    assert_eq!(
        std::fs::read(companion(broken_path, "-wal")).unwrap(),
        b"BROKEN_WAL"
    );
    assert_eq!(
        std::fs::read(companion(broken_path, "-shm")).unwrap(),
        b"BROKEN_SHM"
    );
}

#[cfg(windows)]
#[tokio::test]
async fn restore_puts_everything_back_when_the_journal_cannot_be_moved() {
    use std::os::windows::fs::OpenOptionsExt;

    let tmp = tempfile::TempDir::new().unwrap();
    let auto_dir = make_auto_dir(tmp.path());
    write_backup(&auto_dir, "synk_2026-01-01_pre-v14.db", "SAUVEGARDE").await;
    let db_path = tmp.path().join("synk.db");
    std::fs::write(&db_path, b"BROKEN_DB").unwrap();
    let wal_path = companion(&db_path, "-wal");
    std::fs::write(&wal_path, b"BROKEN_WAL").unwrap();
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&wal_path)
        .unwrap();

    let result = restore_latest_backup_inner(tmp.path()).await;
    drop(lock);

    let message = result.expect_err("la restauration doit s'arreter si le -wal reste en place");
    assert!(
        message.detail().contains("-wal"),
        "message obtenu : {message}"
    );
    assert_eq!(std::fs::read(&db_path).unwrap(), b"BROKEN_DB");
    assert_eq!(std::fs::read(&wal_path).unwrap(), b"BROKEN_WAL");
    let leftovers: Vec<String> = broken_leftovers(tmp.path())
        .into_iter()
        .filter(|name| name.contains("broken"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "rien ne doit rester deplace : {leftovers:?}"
    );
}

#[tokio::test]
async fn restore_ignores_non_db_files_in_auto_dir() {
    let tmp = tempfile::TempDir::new().unwrap();
    let auto_dir = make_auto_dir(tmp.path());
    std::fs::write(auto_dir.join("notes.txt"), b"irrelevant").unwrap();
    std::fs::write(auto_dir.join("synk.db-wal"), b"wal data").unwrap();

    let result = restore_latest_backup_inner(tmp.path()).await;
    assert!(result.is_err(), "should fail when no .db backup present");
}

#[tokio::test]
async fn restores_the_newest_backup_whatever_its_kind() {
    let tmp = tempfile::TempDir::new().unwrap();
    let auto_dir = make_auto_dir(tmp.path());
    let daily_dir = tmp.path().join("backups").join("daily");
    std::fs::create_dir_all(&daily_dir).unwrap();
    let now = std::time::SystemTime::now();
    let auto = write_backup(&auto_dir, "synk_2026-10-01_pre-v16.db", "AUTO").await;
    set_mtime(&auto, now - std::time::Duration::from_secs(2 * 86400));
    let daily = write_backup(&daily_dir, "synk_2026-10-03_quotidienne.db", "QUOTIDIENNE").await;
    set_mtime(&daily, now - std::time::Duration::from_secs(3600));

    let report = restore_latest_backup_inner(tmp.path()).await.unwrap();

    assert!(
        report.restored_from.contains("daily"),
        "obtenu : {}",
        report.restored_from
    );
    assert_eq!(
        read_marker(&tmp.path().join("synk.db")).await,
        "QUOTIDIENNE"
    );
}
