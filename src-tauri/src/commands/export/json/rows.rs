use crate::error::AppError;
use serde_json::{Map, Value};
use sqlx::sqlite::{Sqlite, SqliteArguments};
use sqlx::{Row, SqliteConnection};

use super::bounds::{Bound, BOUNDS};
use super::engine::{db_err, TableImport};
use super::export::row_to_json;
use super::tables::{ParentRef, TableSpec, EXPORTED};
use crate::validation::{check_f64_opt, check_i64_opt};

static NULL_VALUE: Value = Value::Null;

type Query<'q> = sqlx::query::Query<'q, Sqlite, SqliteArguments>;

fn bind_all<'q>(mut q: Query<'q>, values: &[&'q Value]) -> Query<'q> {
    for v in values {
        q = match v {
            Value::Null => q.bind(None::<String>),
            Value::Bool(b) => q.bind(i64::from(*b)),
            Value::Number(n) => match n.as_i64() {
                Some(i) => q.bind(i),
                None => q.bind(n.as_f64()),
            },
            Value::String(s) => q.bind(s.as_str()),
            other => q.bind(other.to_string()),
        };
    }
    q
}

pub(super) fn data_columns(spec: &TableSpec, columns: &[String]) -> Vec<String> {
    let parent_column = spec.parent.as_ref().map(|p| p.column);
    columns
        .iter()
        .filter(|c| c.as_str() != "id" && Some(c.as_str()) != parent_column)
        .cloned()
        .collect()
}

fn check_bounds(spec: &TableSpec, row: &Map<String, Value>) -> Result<(), AppError> {
    for b in BOUNDS.iter().filter(|b| b.table == spec.table) {
        let value = row.get(b.column);
        match b.bound {
            Bound::F(min, max) => check_f64_opt(b.label, value.and_then(Value::as_f64), min, max)?,
            Bound::I(min, max) => check_i64_opt(b.label, value.and_then(Value::as_i64), min, max)?,
        }
    }
    Ok(())
}

fn legacy_parent_key(
    p: &ParentRef,
    row: &Map<String, Value>,
    data: &Map<String, Value>,
) -> Option<Vec<Value>> {
    let old_id = row.get(p.column)?.as_i64()?;
    let parent_key = EXPORTED.iter().find(|s| s.table == p.parent_table)?.key;
    let parent_row = data
        .get(parent_key)?
        .as_array()?
        .iter()
        .find(|r| r.get("id").and_then(Value::as_i64) == Some(old_id))?;
    Some(
        p.parent_key
            .iter()
            .map(|k| parent_row.get(*k).cloned().unwrap_or(Value::Null))
            .collect(),
    )
}

async fn resolve_parent(
    conn: &mut SqliteConnection,
    p: &ParentRef,
    row: &Map<String, Value>,
    data: &Map<String, Value>,
) -> Result<Option<i64>, AppError> {
    let key_values: Vec<Value> = match row.get(p.field).and_then(Value::as_object) {
        Some(parent) => p
            .parent_key
            .iter()
            .map(|k| parent.get(*k).cloned().unwrap_or(Value::Null))
            .collect(),
        None => match legacy_parent_key(p, row, data) {
            Some(values) => values,
            None => return Ok(None),
        },
    };
    let clauses: Vec<String> = p
        .parent_key
        .iter()
        .map(|k| format!("\"{k}\" IS ?"))
        .collect();
    let sql = format!(
        "SELECT id FROM {} WHERE {} LIMIT 1",
        p.parent_table,
        clauses.join(" AND ")
    );
    let refs: Vec<&Value> = key_values.iter().collect();
    let found = bind_all(sqlx::query(sqlx::AssertSqlSafe(sql)), &refs)
        .fetch_optional(&mut *conn)
        .await
        .map_err(db_err)?;
    found
        .map(|r| r.try_get::<i64, _>(0).map_err(db_err))
        .transpose()
}

async fn already_there(
    conn: &mut SqliteConnection,
    spec: &TableSpec,
    names: &[&str],
    values: &[&Value],
) -> Result<bool, AppError> {
    let mut clauses = Vec::new();
    let mut binds = Vec::new();
    for k in spec.dedup {
        let Some(pos) = names.iter().position(|n| n == k) else {
            return Ok(false);
        };
        clauses.push(format!("\"{k}\" IS ?"));
        binds.push(values[pos]);
    }
    if clauses.is_empty() {
        return Ok(false);
    }
    let sql = format!(
        "SELECT 1 FROM {} WHERE {} LIMIT 1",
        spec.table,
        clauses.join(" AND ")
    );
    Ok(bind_all(sqlx::query(sqlx::AssertSqlSafe(sql)), &binds)
        .fetch_optional(&mut *conn)
        .await
        .map_err(db_err)?
        .is_some())
}

async fn fill_profile(
    conn: &mut SqliteConnection,
    names: &[&str],
    values: &[&Value],
    t: &mut TableImport,
) -> Result<(), AppError> {
    sqlx::query("INSERT OR IGNORE INTO user_profile (id) VALUES (1)")
        .execute(&mut *conn)
        .await
        .map_err(db_err)?;
    let current = row_to_json(
        &sqlx::query("SELECT * FROM user_profile WHERE id = 1")
            .fetch_one(&mut *conn)
            .await
            .map_err(db_err)?,
    );
    let mut sets = Vec::new();
    let mut binds = Vec::new();
    for (name, value) in names.iter().zip(values) {
        if !value.is_null() && current.get(*name).is_none_or(Value::is_null) {
            sets.push(format!("\"{name}\" = ?"));
            binds.push(*value);
        }
    }
    if sets.is_empty() {
        t.ignored += 1;
        return Ok(());
    }
    let sql = format!("UPDATE user_profile SET {} WHERE id = 1", sets.join(", "));
    bind_all(sqlx::query(sqlx::AssertSqlSafe(sql)), &binds)
        .execute(&mut *conn)
        .await
        .map_err(db_err)?;
    t.updated += 1;
    Ok(())
}

fn is_legacy_external_id(name: &str, value: &Value) -> bool {
    name.starts_with("withings_") && value.as_i64() == Some(0)
}

async fn source_is_known(conn: &mut SqliteConnection, source: &Value) -> Result<bool, AppError> {
    let Some(id) = source.as_str() else {
        return Ok(false);
    };
    let found: Option<i64> = sqlx::query_scalar("SELECT 1 FROM sources WHERE id = ?1")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(db_err)?;
    Ok(found.is_some())
}

pub(super) async fn import_row(
    conn: &mut SqliteConnection,
    spec: &TableSpec,
    columns: &[String],
    row: &Map<String, Value>,
    data: &Map<String, Value>,
    legacy: bool,
    t: &mut TableImport,
) -> Result<(), AppError> {
    check_bounds(spec, row).map_err(|e| AppError::Internal(format!("{} : {e}", spec.key)))?;
    let mut names: Vec<&str> = Vec::new();
    let mut values: Vec<&Value> = Vec::new();
    for c in data_columns(spec, columns).iter() {
        if let Some((k, v)) = row.get_key_value(c.as_str()) {
            names.push(k.as_str());
            values.push(if legacy && is_legacy_external_id(k, v) {
                &NULL_VALUE
            } else {
                v
            });
        }
    }
    let default_source = Value::from("manual");
    if columns.iter().any(|c| c == "source") && !row.contains_key("source") {
        names.push("source");
        values.push(&default_source);
    }
    if let Some(pos) = names.iter().position(|n| *n == "source") {
        if !source_is_known(conn, values[pos]).await? {
            t.ignored += 1;
            return Ok(());
        }
    }
    let parent_id: Value;
    if let Some(p) = &spec.parent {
        match resolve_parent(conn, p, row, data).await? {
            Some(id) => {
                parent_id = Value::from(id);
                names.push(p.column);
                values.push(&parent_id);
            }
            None => {
                t.orphans += 1;
                return Ok(());
            }
        }
    }
    if spec.fill_only {
        return fill_profile(conn, &names, &values, t).await;
    }
    if names.is_empty() || already_there(conn, spec, &names, &values).await? {
        t.ignored += 1;
        return Ok(());
    }
    let quoted: Vec<String> = names.iter().map(|n| format!("\"{n}\"")).collect();
    let sql = format!(
        "INSERT OR IGNORE INTO {} ({}) VALUES ({})",
        spec.table,
        quoted.join(", "),
        vec!["?"; names.len()].join(", ")
    );
    let done = bind_all(sqlx::query(sqlx::AssertSqlSafe(sql)), &values)
        .execute(&mut *conn)
        .await
        .map_err(db_err)?;
    if done.rows_affected() == 1 {
        t.inserted += 1;
    } else {
        t.ignored += 1;
    }
    Ok(())
}
