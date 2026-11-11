use sqlx::sqlite::SqlitePool;

pub async fn optimize_on_open(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA optimize=0x10002")
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn optimize(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA optimize")
        .execute(pool)
        .await
        .map(|_| ())
}
