use sqlx::sqlite::SqlitePool;

use super::legacy::{migrate_legacy_data_to_v2, table_exists};

pub async fn run_post_ready_seed(pool: &SqlitePool) {
    if let Err(e) = migrate_legacy_data_to_v2(pool).await {
        log::warn!("Post-ready legacy data migration failed: {e}");
    }
    seed_sync_state(pool).await;
}

async fn seed_sync_state(pool: &SqlitePool) {
    // "metrics" : curseur historique (migration 010), conserve pour compat ;
    // plus mis a jour depuis A-01 (sync_body_measures alimente `metrics` directement).
    if let Err(e) = crate::sources::store::seed_cursors(
        pool,
        crate::sources::SourceId::WITHINGS,
        &[
            "measures", "activity", "sleep", "heart", "workouts", "devices", "metrics",
        ],
    )
    .await
    {
        log::warn!("[db] curseurs Withings non initialises : {e}");
    }
}

pub(super) async fn seed_source_definitions(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    for source in crate::sources::all_sources() {
        for d in source.metric_definitions() {
            sqlx::query(
                "INSERT OR IGNORE INTO metric_definitions \
                 (slug, label, unit, category, withings_type, is_builtin, sort_order, decimals, min_value, max_value) \
                 VALUES (?1, ?2, ?3, ?4, NULL, 1, ?5, ?6, ?7, ?8)",
            )
            .bind(d.slug)
            .bind(d.label)
            .bind(d.unit)
            .bind(d.category)
            .bind(d.sort_order)
            .bind(d.decimals)
            .bind(d.min_value)
            .bind(d.max_value)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

// Seed des types built-in depuis le catalogue partage (src/features/bodyProfile/catalog.json).
// Cible `measurement_types_legacy` (la table reelle) car la vue `measurement_types`
// n'est pas inserable. Appele APRES `migrate_to_v2_schema` : avant le renommage,
// la table n'a pas encore son nom `_legacy` sur une base existante.
// Idempotent : `slug` est UNIQUE -> INSERT OR IGNORE, puis UPDATE des metadonnees
// pour que le catalogue reste la source de verite (label, unite, nature, ordre).
pub(super) async fn seed_body_catalog(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    if !table_exists(pool, "measurement_types_legacy").await? {
        return Ok(());
    }
    let mut tx = pool.begin().await?;
    for m in crate::body_catalog::catalog() {
        let result = sqlx::query(
            "INSERT OR IGNORE INTO measurement_types_legacy \
             (slug, label, unit, category, sort_order, is_builtin) \
             VALUES (?1, ?2, 'cm', ?3, ?4, 1)",
        )
        .bind(&m.slug)
        .bind(&m.label)
        .bind(&m.nature)
        .bind(m.order)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            let existing_is_builtin: Option<i64> = sqlx::query_scalar(
                "SELECT is_builtin FROM measurement_types_legacy WHERE slug = ?1",
            )
            .bind(&m.slug)
            .fetch_optional(&mut *tx)
            .await?;
            if existing_is_builtin == Some(0) {
                log::warn!(
                    "seed_body_catalog: slug '{}' du catalogue entre en collision avec un type personnalise existant, seeding ignore pour ce slug",
                    m.slug
                );
                continue;
            }
        }

        sqlx::query(
            "UPDATE measurement_types_legacy \
             SET label = ?2, unit = 'cm', category = ?3, sort_order = ?4, is_builtin = 1 \
             WHERE slug = ?1 AND is_builtin = 1",
        )
        .bind(&m.slug)
        .bind(&m.label)
        .bind(&m.nature)
        .bind(m.order)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
