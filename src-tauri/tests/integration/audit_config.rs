use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use synk_lib::services::audit::{
    get_audit_purge_config, set_audit_purge_config, should_auto_purge, AuditMetrics,
    AuditPurgeConfig,
};

async fn setup_settings_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            theme TEXT NOT NULL DEFAULT 'dark',
            language TEXT NOT NULL DEFAULT 'fr',
            audit_retention_days INTEGER NOT NULL DEFAULT 180,
            audit_max_entries INTEGER NOT NULL DEFAULT 10000,
            audit_auto_purge_enabled INTEGER NOT NULL DEFAULT 1
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO settings (id) VALUES (1)")
        .execute(&pool)
        .await
        .unwrap();
    pool
}

fn make_metrics(total: i64, oldest: Option<&str>) -> AuditMetrics {
    AuditMetrics {
        total_count: total,
        insert_count: 0,
        delete_count: 0,
        oldest_at: oldest.map(String::from),
        newest_at: None,
    }
}

#[test]
fn test_should_auto_purge_when_count_exceeds_max() {
    let metrics = make_metrics(15000, Some("2026-04-01 00:00:00"));
    let config = AuditPurgeConfig {
        retention_days: 180,
        max_entries: 10000,
        enabled: true,
    };
    assert_eq!(
        should_auto_purge(&metrics, &config, "2026-05-07 00:00:00"),
        Some(180)
    );
}

#[test]
fn test_should_auto_purge_when_oldest_exceeds_retention() {
    let metrics = make_metrics(100, Some("2025-01-01 00:00:00"));
    let config = AuditPurgeConfig {
        retention_days: 180,
        max_entries: 10000,
        enabled: true,
    };
    assert_eq!(
        should_auto_purge(&metrics, &config, "2026-05-07 00:00:00"),
        Some(180)
    );
}

#[test]
fn test_should_not_auto_purge_when_disabled() {
    let metrics = make_metrics(50000, Some("2024-01-01 00:00:00"));
    let config = AuditPurgeConfig {
        retention_days: 180,
        max_entries: 10000,
        enabled: false,
    };
    assert_eq!(
        should_auto_purge(&metrics, &config, "2026-05-07 00:00:00"),
        None
    );
}

#[test]
fn test_should_not_auto_purge_when_within_limits() {
    let metrics = make_metrics(100, Some("2026-04-01 00:00:00"));
    let config = AuditPurgeConfig {
        retention_days: 180,
        max_entries: 10000,
        enabled: true,
    };
    assert_eq!(
        should_auto_purge(&metrics, &config, "2026-05-07 00:00:00"),
        None
    );
}

#[tokio::test]
async fn test_get_audit_purge_config_returns_defaults() {
    let pool = setup_settings_pool().await;
    let config = get_audit_purge_config(&pool).await.unwrap();
    assert_eq!(config.retention_days, 180);
    assert_eq!(config.max_entries, 10000);
    assert!(config.enabled);
}

#[tokio::test]
async fn test_set_audit_purge_config_persists_values() {
    let pool = setup_settings_pool().await;
    let new_config = AuditPurgeConfig {
        retention_days: 90,
        max_entries: 5000,
        enabled: false,
    };
    set_audit_purge_config(&pool, new_config.clone())
        .await
        .unwrap();
    let read_back = get_audit_purge_config(&pool).await.unwrap();
    assert_eq!(read_back, new_config);
}

#[tokio::test]
async fn test_set_audit_purge_config_rejects_negative_retention() {
    let pool = setup_settings_pool().await;
    let bad = AuditPurgeConfig {
        retention_days: -1,
        max_entries: 10000,
        enabled: true,
    };
    let result = set_audit_purge_config(&pool, bad).await;
    assert!(result.is_err());
}
