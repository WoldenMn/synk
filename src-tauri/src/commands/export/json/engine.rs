use serde_json::{Map, Value};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection, SqlitePool};

use super::export::row_to_json;
use super::rows::{data_columns, import_row};
use super::tables::{ParentRef, EXPORTED};
use crate::error::AppError;

pub const EXPORT_FORMAT: u64 = 2;

#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct TableImport {
    pub key: String,
    pub inserted: u64,
    pub ignored: u64,
    pub updated: u64,
    pub orphans: u64,
}

#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct ImportReport {
    pub tables: Vec<TableImport>,
}

impl ImportReport {
    pub fn inserted(&self) -> u64 {
        self.tables.iter().map(|t| t.inserted).sum()
    }

    pub fn ignored(&self) -> u64 {
        self.tables.iter().map(|t| t.ignored).sum()
    }
}

pub(super) fn db_err(e: sqlx::Error) -> AppError {
    AppError::from(e)
}

async fn table_columns(
    conn: &mut SqliteConnection,
    table: &'static str,
) -> Result<Vec<String>, AppError> {
    let rows = sqlx::query(sqlx::AssertSqlSafe(format!("PRAGMA table_info({table})")))
        .fetch_all(&mut *conn)
        .await
        .map_err(db_err)?;
    if rows.is_empty() {
        return Err(db_err(sqlx::Error::Protocol(format!(
            "table {table} absente du schema"
        ))));
    }
    rows.iter()
        .map(|r| r.try_get::<String, _>("name").map_err(db_err))
        .collect()
}

fn nest_parent(row: Value, parent: Option<&ParentRef>) -> Value {
    let (Some(p), Value::Object(mut map)) = (parent, row) else {
        return Value::Null;
    };
    let mut nested = Map::new();
    for k in p.parent_key {
        let flat = format!("{}.{}", p.field, k);
        nested.insert((*k).to_string(), map.remove(&flat).unwrap_or(Value::Null));
    }
    map.insert(p.field.to_string(), Value::Object(nested));
    Value::Object(map)
}

pub async fn export_value(pool: &SqlitePool) -> Result<Value, AppError> {
    let mut conn = pool.acquire().await.map_err(db_err)?;
    let mut data = Map::new();
    data.insert("format".into(), Value::from(EXPORT_FORMAT));
    for spec in EXPORTED {
        let columns = data_columns(spec, &table_columns(&mut conn, spec.table).await?);
        let mut select: Vec<String> = columns.iter().map(|c| format!("t.\"{c}\"")).collect();
        if let Some(p) = &spec.parent {
            for k in p.parent_key {
                select.push(format!(
                    "(SELECT p.\"{k}\" FROM {} p WHERE p.id = t.\"{}\") AS \"{}.{}\"",
                    p.parent_table, p.column, p.field, k
                ));
            }
        }
        let sql = format!(
            "SELECT {} FROM {} t ORDER BY t.\"{}\", t.rowid",
            select.join(", "),
            spec.table,
            spec.order_by
        );
        let rows: Vec<SqliteRow> = sqlx::query(sqlx::AssertSqlSafe(sql))
            .fetch_all(&mut *conn)
            .await
            .map_err(db_err)?;
        let values = rows
            .iter()
            .map(|r| {
                let row = row_to_json(r);
                match &spec.parent {
                    Some(_) => nest_parent(row, spec.parent.as_ref()),
                    None => row,
                }
            })
            .collect();
        data.insert(spec.key.into(), Value::Array(values));
    }
    Ok(Value::Object(data))
}

pub async fn import_value(
    pool: &SqlitePool,
    data: &Map<String, Value>,
) -> Result<ImportReport, AppError> {
    let legacy = !data.contains_key("format");
    let mut tx = pool.begin().await.map_err(db_err)?;
    let mut report = ImportReport::default();
    for spec in EXPORTED {
        let Some(rows) = data.get(spec.key).and_then(Value::as_array) else {
            continue;
        };
        let columns = table_columns(&mut tx, spec.table).await?;
        let mut t = TableImport {
            key: spec.key.to_string(),
            ..TableImport::default()
        };
        for row in rows {
            let obj = row.as_object().ok_or_else(|| {
                AppError::Validation(format!("{} : ligne qui n'est pas un objet JSON", spec.key))
            })?;
            import_row(&mut tx, spec, &columns, obj, data, legacy, &mut t).await?;
        }
        report.tables.push(t);
    }
    tx.commit().await.map_err(db_err)?;
    Ok(report)
}
