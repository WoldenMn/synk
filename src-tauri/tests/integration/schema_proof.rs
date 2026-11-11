use sqlx::{Row, SqlitePool};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

struct Snapshot {
    integrity: String,
    foreign_keys: Vec<String>,
    counts: BTreeMap<String, i64>,
    columns: BTreeMap<String, Vec<String>>,
    version: i64,
}

async fn snapshot(pool: &SqlitePool) -> Snapshot {
    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(pool)
        .await
        .unwrap();
    let foreign_keys: Vec<String> = sqlx::query("PRAGMA foreign_key_check")
        .fetch_all(pool)
        .await
        .unwrap()
        .iter()
        .map(|r| format!("{}:{}", r.get::<String, _>(0), r.get::<i64, _>(1)))
        .collect();
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    let mut counts = BTreeMap::new();
    let mut columns = BTreeMap::new();
    for t in &tables {
        let n: i64 =
            sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM \"{t}\"")))
                .fetch_one(pool)
                .await
                .unwrap();
        counts.insert(t.clone(), n);
        let cols: Vec<String> =
            sqlx::query(sqlx::AssertSqlSafe(format!("PRAGMA table_info(\"{t}\")")))
                .fetch_all(pool)
                .await
                .unwrap()
                .iter()
                .map(|r| r.get::<String, _>("name"))
                .collect();
        columns.insert(t.clone(), cols);
    }
    let version: i64 = sqlx::query_scalar("SELECT MAX(version) FROM schema_version")
        .fetch_one(pool)
        .await
        .unwrap();
    Snapshot {
        integrity,
        foreign_keys,
        counts,
        columns,
        version,
    }
}

async fn content_hash(pool: &SqlitePool, table: &str, cols: &[String]) -> (usize, u64) {
    let list = cols
        .iter()
        .map(|c| format!("CAST(\"{c}\" AS TEXT)"))
        .collect::<Vec<_>>()
        .join(", ");
    let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {list} FROM \"{table}\" ORDER BY 1"
    )))
    .fetch_all(pool)
    .await
    .unwrap();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for r in &rows {
        for i in 0..cols.len() {
            r.get::<Option<String>, _>(i).hash(&mut hasher);
        }
    }
    (rows.len(), hasher.finish())
}

async fn all_series(pool: &SqlitePool) -> BTreeMap<String, String> {
    let slugs: Vec<String> =
        sqlx::query_scalar("SELECT slug FROM metric_definitions ORDER BY slug")
            .fetch_all(pool)
            .await
            .unwrap();
    let mut out = BTreeMap::new();
    for slug in slugs {
        let series = synk_lib::metric_repository::read_metric_series(
            pool,
            &slug,
            "2000-01-01",
            "2100-01-01",
        )
        .await
        .unwrap();
        let latest = synk_lib::metric_repository::read_metric_latest(pool, &slug)
            .await
            .unwrap();
        out.insert(
            slug,
            format!(
                "{}|{}",
                serde_json::to_string(&series).unwrap(),
                serde_json::to_string(&latest).unwrap()
            ),
        );
    }
    out
}

/// Preuve d'une migration sur une copie de la base de dev : `SYNK_PROOF_COPY`
/// est un dossier qui contient `synk.db`. Jamais la vraie base. Avec
/// `SYNK_PROOF_SEED_TOKEN=1`, un jeton de test scelle par la cle `synk-dev` est
/// pose dans la copie avant la migration, puis relu apres.
#[tokio::test]
#[ignore]
async fn proof_on_a_copy() {
    let dir = PathBuf::from(std::env::var("SYNK_PROOF_COPY").expect("SYNK_PROOF_COPY"));
    let db = dir.join("synk.db");
    let seed_token = std::env::var("SYNK_PROOF_SEED_TOKEN").is_ok();
    let dev_vault = synk_lib::crypto::CryptoVault::init("synk-dev").unwrap();
    let before_pool = SqlitePool::connect(&format!("sqlite:{}?mode=rw", db.display()))
        .await
        .unwrap();
    if seed_token {
        sqlx::query(
            "INSERT OR REPLACE INTO withings_auth (id, access_token, refresh_token, expires_at, token_type) VALUES (1, ?1, ?2, 1893456000, 'Bearer')",
        )
        .bind(dev_vault.seal_str("jeton-acces-essai").unwrap())
        .bind(dev_vault.seal_str("jeton-refresh-essai").unwrap())
        .execute(&before_pool)
        .await
        .unwrap();
    }
    let before = snapshot(&before_pool).await;
    let mut hashes_before = BTreeMap::new();
    for (t, cols) in &before.columns {
        hashes_before.insert(t.clone(), content_hash(&before_pool, t, cols).await);
    }
    let views_before: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'view' ORDER BY name")
            .fetch_all(&before_pool)
            .await
            .unwrap();
    let series_before = all_series(&before_pool).await;
    for v in &views_before {
        let cols: Vec<String> =
            sqlx::query(sqlx::AssertSqlSafe(format!("PRAGMA table_info(\"{v}\")")))
                .fetch_all(&before_pool)
                .await
                .unwrap()
                .iter()
                .map(|r| r.get::<String, _>("name"))
                .collect();
        hashes_before.insert(
            format!("vue {v}"),
            content_hash(&before_pool, v, &cols).await,
        );
    }
    let before_cols_views: BTreeMap<String, Vec<String>> = {
        let mut m = BTreeMap::new();
        for v in &views_before {
            let cols: Vec<String> =
                sqlx::query(sqlx::AssertSqlSafe(format!("PRAGMA table_info(\"{v}\")")))
                    .fetch_all(&before_pool)
                    .await
                    .unwrap()
                    .iter()
                    .map(|r| r.get::<String, _>("name"))
                    .collect();
            m.insert(v.clone(), cols);
        }
        m
    };
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .fetch_all(&before_pool)
        .await
        .unwrap();
    before_pool.close().await;

    let backup = synk_lib::boot::backup_before_migration(&db, &dir)
        .await
        .expect("sauvegarde avant migration")
        .expect("une migration est due, la sauvegarde doit exister");
    println!("sauvegarde : {}", backup.display());
    let saved = {
        let pool = SqlitePool::connect(&format!("sqlite:{}?mode=ro", backup.display()))
            .await
            .unwrap();
        let s = snapshot(&pool).await;
        pool.close().await;
        s
    };
    println!(
        "SAUVEGARDE version={} integrity={} comptes egaux a avant : {}",
        saved.version,
        saved.integrity,
        saved.counts == before.counts
    );
    assert_eq!(saved.counts, before.counts);

    let pool = synk_lib::db::init_pool(db.clone()).await.unwrap();
    let after = snapshot(&pool).await;
    println!(
        "AVANT version={} integrity={} fk={:?}",
        before.version, before.integrity, before.foreign_keys
    );
    println!(
        "APRES version={} integrity={} fk={:?}",
        after.version, after.integrity, after.foreign_keys
    );
    let mut equal = true;
    for (t, n) in &before.counts {
        let m = after.counts.get(t).copied();
        let changed = t != "schema_version" && m != Some(*n);
        println!(
            "table {t} avant={n} apres={m:?}{}",
            if changed { "  <-- change" } else { "" }
        );
        if changed && m.is_some() {
            equal = false;
        }
    }
    let removed: Vec<&String> = before
        .counts
        .keys()
        .filter(|t| !after.counts.contains_key(*t))
        .collect();
    println!("tables devenues vues ou retirees : {removed:?}");
    println!("comptes des tables restantes egaux : {equal}");
    let mut content_equal = true;
    for (t, cols) in &before.columns {
        if t == "schema_version" {
            continue;
        }
        let h_after = content_hash(&pool, t, cols).await;
        let same = hashes_before.get(t) == Some(&h_after);
        println!("contenu {t} : {} lignes, identique : {same}", h_after.0);
        content_equal &= same;
    }
    for (v, cols) in &before_cols_views {
        let h_after = content_hash(&pool, v, cols).await;
        let same = hashes_before.get(&format!("vue {v}")) == Some(&h_after);
        println!("contenu vue {v} : {} lignes, identique : {same}", h_after.0);
        content_equal &= same;
    }
    println!("contenu de toutes les colonnes d'avant identique : {content_equal}");
    let series_after = all_series(&pool).await;
    let series_equal = series_before == series_after;
    println!(
        "{} slugs, get_metric_series et dernier point identiques avant et apres : {series_equal}",
        series_after.len()
    );
    let new_tables: Vec<&String> = after
        .counts
        .keys()
        .filter(|t| !before.counts.contains_key(*t))
        .collect();
    println!("tables neuves : {new_tables:?}");
    if seed_token {
        let stored = synk_lib::sources::store::load_auth(
            &pool,
            &dev_vault,
            synk_lib::sources::SourceId::WITHINGS,
        )
        .await
        .unwrap()
        .expect("jeton recopie");
        println!(
            "jeton recopie dans source_auth, dechiffre par la cle synk-dev : {}",
            stored.access_token.as_str() == "jeton-acces-essai"
                && stored.refresh_token.as_str() == "jeton-refresh-essai"
        );
        assert_eq!(stored.access_token.as_str(), "jeton-acces-essai");
    }
    let (id,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sources")
        .fetch_one(&pool)
        .await
        .unwrap();
    println!("sources : {id}");
    assert!(equal && content_equal && series_equal);
    assert_eq!(after.integrity, "ok");
    assert!(after.foreign_keys.is_empty());
    pool.close().await;
}
