use sqlx::sqlite::SqlitePool;

use super::legacy::migrate_to_v2_schema;
use super::migrations::{run_versioned_migration, InitError};
use super::seed::{seed_body_catalog, seed_source_definitions};

const MIGRATION_001: &str = include_str!("../../migrations/001_init.sql");

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

macro_rules! migration {
    ($version:expr, $name:literal) => {
        Migration {
            version: $version,
            name: $name,
            sql: include_str!(concat!("../../migrations/", $name)),
        }
    };
}

pub const MIGRATIONS: [Migration; 22] = [
    migration!(2, "002_v2.sql"),
    migration!(3, "003_v3.sql"),
    migration!(4, "004_fixes.sql"),
    migration!(5, "005_metrics_unification.sql"),
    migration!(6, "006_consolidation.sql"),
    migration!(7, "007_withings_credentials.sql"),
    migration!(8, "008_audit_purge_settings.sql"),
    migration!(9, "009_metric_definitions_extension.sql"),
    migration!(10, "010_sync_state_metrics.sql"),
    migration!(11, "011_clothing_sizes.sql"),
    migration!(12, "012_error_log.sql"),
    migration!(13, "013_temperature_metric.sql"),
    migration!(14, "014_widen_vital_bounds.sql"),
    migration!(15, "015_profile_identity.sql"),
    migration!(16, "016_clothing_sizes_estimated.sql"),
    migration!(17, "017_index_et_cascades.sql"),
    migration!(18, "018_tables_v2_mortes.sql"),
    migration!(19, "019_sources.sql"),
    migration!(20, "020_cle_source.sql"),
    migration!(21, "021_journal_synchro_source.sql"),
    migration!(22, "022_appareil_d_origine.sql"),
    migration!(23, "023_traces_et_correspondances_csv.sql"),
];

pub const FIRST_LEGACY_NAMED_VERSION: i64 = 19;

pub const LATEST_VERSION: i64 = MIGRATIONS[MIGRATIONS.len() - 1].version;

// MIGRATION_001 est rejouee a chaque demarrage : sur une base existante, ses
// CREATE trouvent leurs objets, et ses INSERT comme ses CREATE INDEX visent des
// tables devenues des vues de retrocompat. Seul un autre echec merite d'etre vu.
pub fn migration_001_replay_is_benign(msg: &str) -> bool {
    msg.contains("because it is a view")
        || msg.contains("views may not be indexed")
        || msg.contains("already exists")
        || msg.contains("duplicate column")
}

// `pub` requis : les tests d'integration dans `tests/` sont compiles comme
// des crates separes, donc `pub(crate)` ne suffirait pas.
pub async fn bootstrap_schema(pool: &SqlitePool) -> Result<(), InitError> {
    bootstrap_schema_up_to(pool, LATEST_VERSION).await
}

pub async fn bootstrap_schema_up_to(pool: &SqlitePool, max_version: i64) -> Result<(), InitError> {
    // MIGRATION_001 est applique en best-effort (split naif par ';') car il ne
    // contient pas d'instructions multi-statement et sert de baseline sur les
    // DB fraiches ou pre-versionnees (avant que schema_version n'existe).
    for statement in MIGRATION_001.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            if let Err(e) = sqlx::query(trimmed).execute(pool).await {
                let msg = e.to_string();
                if migration_001_replay_is_benign(&msg) {
                    log::debug!("[db] MIGRATION_001 rejouee, instruction sans effet : {msg}");
                } else {
                    log::warn!("[db] MIGRATION_001 rejouee, instruction en echec : {msg}");
                }
            }
        }
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_version (\
            version INTEGER PRIMARY KEY,\
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))\
        )",
    )
    .execute(pool)
    .await?;

    let mut renamed = false;
    for migration in MIGRATIONS.iter().filter(|m| m.version <= max_version) {
        if !renamed && migration.version >= FIRST_LEGACY_NAMED_VERSION {
            migrate_to_v2_schema(pool).await?;
            renamed = true;
        }
        run_versioned_migration(pool, migration.version, migration.sql).await?;
    }
    migrate_to_v2_schema(pool).await?;
    seed_body_catalog(pool).await?;
    seed_source_definitions(pool).await?;

    Ok(())
}
