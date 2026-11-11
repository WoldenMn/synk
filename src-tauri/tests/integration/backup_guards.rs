use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::path::Path;
use synk_lib::services::backup::{copy_database_safely, create_backup_in, verify_backup};

async fn source_with_a_table(db_path: &Path) -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&format!("sqlite:{}?mode=rwc", db_path.display()))
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
    pool
}

#[tokio::test]
async fn two_backups_to_the_same_target_never_share_the_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("synk.db");
    let pool = source_with_a_table(&db_path).await;
    let backup_path = dir.path().join("backup.db");

    let (a, b) = tokio::join!(
        copy_database_safely(&db_path, &backup_path, Some(&pool)),
        copy_database_safely(&db_path, &backup_path, Some(&pool)),
    );
    pool.close().await;

    let outcomes = [a, b];
    let refused: Vec<&synk_lib::error::AppError> =
        outcomes.iter().filter_map(|o| o.as_ref().err()).collect();
    assert_eq!(
        refused.len(),
        1,
        "une seule sauvegarde doit aboutir : {outcomes:?}"
    );
    assert!(
        refused[0].detail().contains("existe déjà"),
        "la seconde doit etre refusee avant d'ecrire : {}",
        refused[0]
    );
    verify_backup(&backup_path).await.unwrap();
    assert!(!dir.path().join("backup.db.partial").exists());
}

#[tokio::test]
async fn verification_rejects_a_copy_without_any_table() {
    let dir = tempfile::tempdir().unwrap();
    let empty = dir.path().join("vide.db");
    std::fs::write(&empty, b"").unwrap();

    let message = verify_backup(&empty)
        .await
        .expect_err("un fichier vide n'est pas une sauvegarde");
    assert!(
        message.detail().contains("aucune table"),
        "message obtenu : {message}"
    );
}

#[tokio::test]
async fn a_manual_backup_without_a_database_is_refused() {
    let dir = tempfile::tempdir().unwrap();

    let error = create_backup_in(dir.path(), None)
        .await
        .expect_err("pas de base, pas de sauvegarde");

    assert_eq!(error.detail(), "Base de données introuvable");
    assert!(!dir.path().join("backups").exists());
}

#[tokio::test]
async fn a_manual_backup_returns_the_path_of_a_verified_copy() {
    let dir = tempfile::tempdir().unwrap();
    let pool = source_with_a_table(&dir.path().join("synk.db")).await;

    let path = create_backup_in(dir.path(), Some(&pool))
        .await
        .expect("sauvegarde manuelle");
    pool.close().await;

    let backup = Path::new(&path);
    assert_eq!(backup.parent(), Some(dir.path().join("backups").as_path()));
    let name = backup.file_name().unwrap().to_string_lossy();
    assert!(
        name.starts_with("synk_") && name.ends_with(".db"),
        "nom inattendu : {name}"
    );
    verify_backup(backup).await.unwrap();
}
