use synk_lib::boot::integrity_check;

#[tokio::test]
async fn a_check_that_cannot_run_is_a_warning_not_a_success() {
    let dir = tempfile::tempdir().unwrap();
    let pool = synk_lib::db::init_pool(dir.path().join("synk.db"))
        .await
        .unwrap();
    pool.close().await;
    let warning = integrity_check(&pool).await.expect("avertissement attendu");
    assert!(
        warning.details.contains("contrôle impossible"),
        "{}",
        warning.details
    );
}

#[tokio::test]
async fn a_damaged_index_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("abimee.db");
    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}?mode=rwc", path.display()))
        .await
        .unwrap();
    sqlx::query("PRAGMA journal_mode=DELETE")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE t (x TEXT)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE INDEX i ON t (x)")
        .execute(&pool)
        .await
        .unwrap();
    for n in 0..2000 {
        sqlx::query("INSERT INTO t (x) VALUES (?)")
            .bind(format!("valeur-{n:05}"))
            .execute(&pool)
            .await
            .unwrap();
    }
    let (root, page_size): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT rootpage FROM sqlite_master WHERE name = 'i'), (SELECT page_size FROM pragma_page_size())",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    pool.close().await;
    let mut bytes = std::fs::read(&path).unwrap();
    let end = (root * page_size) as usize;
    bytes[end - 300..end].fill(0xFF);
    std::fs::write(&path, bytes).unwrap();

    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", path.display()))
        .await
        .unwrap();
    let warning = integrity_check(&pool).await.expect("corruption attendue");
    assert!(!warning.details.is_empty());
}
