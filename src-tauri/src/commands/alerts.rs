use crate::error::AppError;
use crate::validation::{check_f64, check_one_of, check_text};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HealthAlert {
    pub id: i64,
    pub metric_key: String,
    pub condition: String,
    pub threshold: f64,
    pub label: String,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertEvent {
    pub id: i64,
    pub alert_id: i64,
    pub value: f64,
    pub triggered_at: String,
    pub seen: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddAlertInput {
    pub metric_key: String,
    pub condition: String,
    pub threshold: f64,
    pub label: String,
}

impl AddAlertInput {
    pub fn validate(&self) -> Result<(), AppError> {
        check_text("Metrique", &self.metric_key, 40)?;
        check_one_of("Condition", &self.condition, &["above", "below"])?;
        check_f64("Seuil", self.threshold, -1_000_000.0, 1_000_000.0)?;
        check_text("Nom de l'alerte", &self.label, 60)
    }
}

#[tauri::command]
pub async fn get_alerts(pool: State<'_, SqlitePool>) -> Result<Vec<HealthAlert>, AppError> {
    sqlx::query_as::<_, HealthAlert>(
        "SELECT id, metric_key, condition, threshold, label, enabled, created_at \
         FROM health_alerts ORDER BY created_at DESC",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(AppError::from)
}

pub async fn add_alert_inner(pool: &SqlitePool, data: &AddAlertInput) -> Result<i64, AppError> {
    data.validate()?;
    let known: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM metric_definitions WHERE slug = ?1")
        .bind(&data.metric_key)
        .fetch_one(pool)
        .await?;
    if known == 0 {
        return Err(AppError::Validation(format!(
            "Metrique inconnue : {}",
            data.metric_key
        )));
    }
    let result = sqlx::query(
        "INSERT INTO health_alerts (metric_key, condition, threshold, label) \
         VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(&data.metric_key)
    .bind(&data.condition)
    .bind(data.threshold)
    .bind(data.label.trim())
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn add_alert(pool: State<'_, SqlitePool>, data: AddAlertInput) -> Result<i64, AppError> {
    add_alert_inner(pool.inner(), &data).await
}

pub async fn delete_alert_inner(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM health_alerts WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_alert(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    delete_alert_inner(pool.inner(), id).await
}

#[tauri::command]
pub async fn get_alert_events(pool: State<'_, SqlitePool>) -> Result<Vec<AlertEvent>, AppError> {
    sqlx::query_as::<_, AlertEvent>(
        "SELECT id, alert_id, value, triggered_at, seen \
         FROM alert_events ORDER BY triggered_at DESC LIMIT 100",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(AppError::from)
}

#[tauri::command]
pub async fn mark_alerts_seen(pool: State<'_, SqlitePool>) -> Result<(), AppError> {
    sqlx::query("UPDATE alert_events SET seen = 1 WHERE seen = 0")
        .execute(pool.inner())
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn evaluate_alerts(pool: State<'_, SqlitePool>) -> Result<Vec<AlertEvent>, AppError> {
    run_alert_checks(pool.inner()).await
}

pub async fn run_alert_checks(pool: &SqlitePool) -> Result<Vec<AlertEvent>, AppError> {
    let alerts = sqlx::query_as::<_, HealthAlert>(
        "SELECT id, metric_key, condition, threshold, label, enabled, created_at \
         FROM health_alerts WHERE enabled = 1",
    )
    .fetch_all(pool)
    .await?;

    let mut triggered: Vec<AlertEvent> = Vec::new();

    for alert in &alerts {
        let latest_value = crate::services::metrics::select_latest(pool, &alert.metric_key)
            .await?
            .map(|p| p.value);

        if let Some(val) = latest_value {
            let should_trigger = match alert.condition.as_str() {
                "above" => val > alert.threshold,
                "below" => val < alert.threshold,
                _ => false,
            };

            if should_trigger {
                let utc = chrono::Utc::now();
                let now = utc.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                let today = utc.format("%Y-%m-%d").to_string();
                let today_start = format!("{today}T00:00:00Z");
                let today_end = format!("{today}T23:59:59Z");

                let existing: Option<(i64,)> = sqlx::query_as(
                    "SELECT id FROM alert_events \
                     WHERE alert_id = ?1 AND triggered_at >= ?2 AND triggered_at <= ?3",
                )
                .bind(alert.id)
                .bind(&today_start)
                .bind(&today_end)
                .fetch_optional(pool)
                .await?;

                if existing.is_none() {
                    let result = sqlx::query(
                        "INSERT INTO alert_events (alert_id, value, triggered_at) \
                         VALUES (?1, ?2, ?3)",
                    )
                    .bind(alert.id)
                    .bind(val)
                    .bind(&now)
                    .execute(pool)
                    .await?;

                    triggered.push(AlertEvent {
                        id: result.last_insert_rowid(),
                        alert_id: alert.id,
                        value: val,
                        triggered_at: now,
                        seen: false,
                    });
                }
            }
        }
    }

    Ok(triggered)
}
