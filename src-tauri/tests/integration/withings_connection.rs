use sqlx::SqlitePool;
use synk_lib::sources::withings::auth::is_connected_inner;

async fn bootstrapped_pool(dir: &tempfile::TempDir) -> SqlitePool {
    synk_lib::db::init_pool(dir.path().join("synk.db"))
        .await
        .expect("base temporaire initialisee")
}

async fn insert_auth(pool: &SqlitePool, refresh: &str, expires_in_secs: i64) {
    sqlx::query(
        "INSERT INTO source_auth (source_id, access_token, refresh_token, expires_at, token_type) \
         VALUES ('withings', 'enc:v1:acces', ?1, unixepoch('now') + ?2, 'Bearer')",
    )
    .bind(refresh)
    .bind(expires_in_secs)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn expired_access_token_with_refresh_token_counts_as_connected() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    insert_auth(&pool, "enc:v1:rafraichissement", -3600).await;
    assert!(is_connected_inner(&pool).await.unwrap());
}

#[tokio::test]
async fn no_auth_row_is_not_connected() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    assert!(!is_connected_inner(&pool).await.unwrap());
}

#[tokio::test]
async fn empty_refresh_token_is_not_connected() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    insert_auth(&pool, "", 3600).await;
    assert!(!is_connected_inner(&pool).await.unwrap());
}

#[tokio::test]
async fn disconnecting_forgets_the_tokens_and_restarts_every_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let pool = bootstrapped_pool(&dir).await;
    insert_auth(&pool, "enc:v1:rafraichissement", 3600).await;
    sqlx::query("UPDATE source_state SET last_sync_at = 1700000000")
        .execute(&pool)
        .await
        .unwrap();

    synk_lib::sources::withings::auth::disconnect_inner(&pool)
        .await
        .unwrap();

    assert!(!is_connected_inner(&pool).await.unwrap());
    let cursors: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sync_state WHERE last_sync_at != 0")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cursors, 0, "la prochaine connexion repart du debut");
    let cursor_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_state")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(cursor_rows >= 6, "les curseurs restent, remis a zero");
}

#[test]
fn the_authorization_url_carries_the_encoded_client_state_and_scope() {
    let url = synk_lib::sources::withings::auth::authorization_url("client id&1", "etat/x");

    assert!(url.starts_with("https://account.withings.com/oauth2_user/authorize2?"));
    assert!(url.contains("response_type=code"));
    assert!(url.contains("client_id=client%20id%261"), "{url}");
    assert!(url.contains("state=etat%2Fx"), "{url}");
    assert!(
        url.contains("scope=user.info%2Cuser.metrics%2Cuser.activity"),
        "{url}"
    );
    assert!(url.contains("redirect_uri=http%3A%2F%2F"), "{url}");
}
