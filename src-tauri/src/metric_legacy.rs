use crate::metric_repository::{LegacyTable, MetricPoint};
use sqlx::{Pool, Sqlite};

pub(crate) fn body_composition_column(slug: &str) -> Option<&'static str> {
    match slug {
        "weight" => Some("weight"),
        "fat_ratio" => Some("fat_ratio"),
        "fat_mass" => Some("fat_mass"),
        "fat_free_mass" => Some("fat_free_mass"),
        "muscle_mass" => Some("muscle_mass"),
        "bone_mass" => Some("bone_mass"),
        "hydration" => Some("hydration"),
        "bmi" => Some("bmi"),
        _ => None,
    }
}

pub(crate) struct LegacyQuery {
    pub select: String,
    pub from_where: String,
    pub order_col: &'static str,
    pub binds_slug: bool,
}

pub(crate) fn legacy_query(table: LegacyTable, slug: &str) -> Option<LegacyQuery> {
    let (select, from_where, order_col, binds_slug) = match table {
        LegacyTable::BodyMeasures => {
            let col = body_composition_column(slug)?;
            (
                format!("measured_at AS measured_at, {col} AS value, source"),
                format!("body_measures_legacy WHERE {col} IS NOT NULL"),
                "measured_at",
                false,
            )
        }
        LegacyTable::ManualMeasurements => (
            "mm.measured_at AS measured_at, mm.value AS value, 'manual' AS source".to_string(),
            "manual_measurements_legacy mm JOIN measurement_types_legacy mt ON mt.id = mm.measurement_type_id WHERE mt.slug = ?"
                .to_string(),
            "mm.measured_at",
            true,
        ),
        LegacyTable::Vitals => (
            "v.measured_at AS measured_at, v.value AS value, v.source AS source".to_string(),
            "vitals_legacy v WHERE v.vital_type = ?".to_string(),
            "v.measured_at",
            true,
        ),
        LegacyTable::BloodPressure => {
            let col = match slug {
                "systolic" | "diastolic" | "heart_rate" => slug,
                _ => return None,
            };
            (
                format!("measured_at AS measured_at, CAST({col} AS REAL) AS value, source"),
                format!("blood_pressure WHERE {col} IS NOT NULL"),
                "measured_at",
                false,
            )
        }
        LegacyTable::SleepRecords => (
            "date AS measured_at, CAST(sleep_score AS REAL) AS value, source".to_string(),
            "sleep_records WHERE sleep_score IS NOT NULL".to_string(),
            "date",
            false,
        ),
        LegacyTable::Activity => (
            "date AS measured_at, CAST(steps AS REAL) AS value, source".to_string(),
            "activity WHERE steps IS NOT NULL".to_string(),
            "date",
            false,
        ),
    };
    Some(LegacyQuery {
        select,
        from_where,
        order_col,
        binds_slug,
    })
}

pub fn series_sql(table: LegacyTable, slug: &str) -> Option<String> {
    let q = legacy_query(table, slug)?;
    Some(format!(
        "SELECT {} FROM {} AND {col} >= ? AND {col} < ? ORDER BY {col} ASC",
        q.select,
        q.from_where,
        col = q.order_col
    ))
}

pub(crate) async fn legacy_series(
    pool: &Pool<Sqlite>,
    table: LegacyTable,
    slug: &str,
    from: &str,
    to: &str,
) -> Result<Vec<MetricPoint>, sqlx::Error> {
    let Some(q) = legacy_query(table, slug) else {
        return Ok(Vec::new());
    };
    let Some(sql) = series_sql(table, slug) else {
        return Ok(Vec::new());
    };
    let mut query = sqlx::query_as::<_, MetricPoint>(sqlx::AssertSqlSafe(sql));
    if q.binds_slug {
        query = query.bind(slug.to_string());
    }
    let points = query
        .bind(from.to_string())
        .bind(to.to_string())
        .fetch_all(pool)
        .await?;
    Ok(points)
}

pub(crate) async fn legacy_latest(
    pool: &Pool<Sqlite>,
    table: LegacyTable,
    slug: &str,
    before: Option<&str>,
) -> Result<Option<MetricPoint>, sqlx::Error> {
    let Some(q) = legacy_query(table, slug) else {
        return Ok(None);
    };
    let bound = before.map_or(String::new(), |_| format!(" AND {} < ?", q.order_col));
    let sql = format!(
        "SELECT {} FROM {}{bound} ORDER BY {} DESC LIMIT 1",
        q.select, q.from_where, q.order_col
    );
    let mut query = sqlx::query_as::<_, MetricPoint>(sqlx::AssertSqlSafe(sql));
    if q.binds_slug {
        query = query.bind(slug.to_string());
    }
    if let Some(day) = before {
        query = query.bind(day.to_string());
    }
    query.fetch_optional(pool).await
}
