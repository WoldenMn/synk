use sqlx::sqlite::SqlitePool;

pub(super) async fn table_exists(pool: &SqlitePool, name: &str) -> Result<bool, sqlx::Error> {
    let row: Option<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' AND name = ?")
            .bind(name)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}

async fn rename_legacy_if_exists(
    pool: &SqlitePool,
    old: &str,
    new: &str,
) -> Result<(), sqlx::Error> {
    if table_exists(pool, old).await? && !table_exists(pool, new).await? {
        let sql = format!("ALTER TABLE {} RENAME TO {}", old, new);
        sqlx::query(sqlx::AssertSqlSafe(sql)).execute(pool).await?;
    }
    Ok(())
}

// Migration V2 : echec fort via `?` pour eviter un etat incoherent.
// Renommage conditionnel des tables V1 en `*_legacy` + creation des vues
// de retrocompatibilite. Toute erreur ici interrompt le bootstrap.
pub(super) async fn migrate_to_v2_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    rename_legacy_if_exists(pool, "body_measures", "body_measures_legacy").await?;
    rename_legacy_if_exists(pool, "manual_measurements", "manual_measurements_legacy").await?;
    rename_legacy_if_exists(pool, "vitals", "vitals_legacy").await?;
    rename_legacy_if_exists(pool, "measurement_types", "measurement_types_legacy").await?;

    for (legacy, view) in [
        ("body_measures_legacy", "body_measures"),
        ("manual_measurements_legacy", "manual_measurements"),
        ("vitals_legacy", "vitals"),
        ("measurement_types_legacy", "measurement_types"),
    ] {
        if table_exists(pool, legacy).await? {
            let sql = format!(
                "CREATE VIEW IF NOT EXISTS {} AS SELECT * FROM {}",
                view, legacy
            );
            sqlx::query(sqlx::AssertSqlSafe(sql)).execute(pool).await?;
        }
    }

    Ok(())
}

pub(super) async fn migrate_legacy_data_to_v2(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    if table_exists(pool, "measurement_types_legacy").await? {
        sqlx::query(
            "INSERT OR IGNORE INTO metric_definitions (slug, label, unit, category, is_builtin, sort_order, decimals, created_at)
             SELECT mt.slug, mt.label, mt.unit, 'custom', 0, COALESCE(mt.sort_order, 100), 2, COALESCE(mt.created_at, datetime('now'))
             FROM measurement_types_legacy mt
             WHERE mt.is_builtin = 0",
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
