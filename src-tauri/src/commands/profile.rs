use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserProfile {
    pub id: i64,
    pub age: Option<i64>,
    pub sex: Option<String>,
    pub height_cm: Option<f64>,
}

#[tauri::command]
pub async fn get_user_profile(pool: State<'_, SqlitePool>) -> Result<UserProfile, AppError> {
    let profile: Option<UserProfile> =
        sqlx::query_as("SELECT id, age, sex, height_cm FROM user_profile WHERE id = 1")
            .fetch_optional(pool.inner())
            .await?;

    Ok(profile.unwrap_or(UserProfile {
        id: 1,
        age: None,
        sex: None,
        height_cm: None,
    }))
}

async fn update_user_profile_inner(
    pool: &SqlitePool,
    age: Option<i64>,
    sex: Option<String>,
    height_cm: Option<f64>,
) -> Result<(), AppError> {
    crate::validation::check_i64_opt("Age", age, 0, 150)?;
    crate::validation::check_f64_opt("Taille", height_cm, 0.0, 300.0)?;
    sqlx::query(
        "INSERT INTO user_profile (id, age, sex, height_cm) VALUES (1, ?1, ?2, ?3) \
         ON CONFLICT(id) DO UPDATE SET age = ?1, sex = ?2, height_cm = ?3",
    )
    .bind(age)
    .bind(&sex)
    .bind(height_cm)
    .execute(pool)
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn update_user_profile(
    pool: State<'_, SqlitePool>,
    age: Option<i64>,
    sex: Option<String>,
    height_cm: Option<f64>,
) -> Result<(), AppError> {
    update_user_profile_inner(pool.inner(), age, sex, height_cm).await
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn pool() -> SqlitePool {
        let p = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap();
        crate::db::bootstrap_schema(&p).await.unwrap();
        p
    }

    #[tokio::test]
    async fn rejects_out_of_range_age() {
        let p = pool().await;
        assert!(update_user_profile_inner(&p, Some(200), None, None)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn rejects_out_of_range_height() {
        let p = pool().await;
        assert!(update_user_profile_inner(&p, None, None, Some(400.0))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn accepts_valid_profile() {
        let p = pool().await;
        assert!(
            update_user_profile_inner(&p, Some(40), Some("male".into()), Some(178.0))
                .await
                .is_ok()
        );
    }
}
