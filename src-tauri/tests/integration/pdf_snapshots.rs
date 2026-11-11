use std::path::PathBuf;

use sqlx::SqlitePool;
use synk_lib::commands::body_profile::{
    add_measurement_session_inner, get_body_sheet_inner, SessionEntry,
};
use synk_lib::commands::export::{export_pdf_to, render_profile_card, render_sheet};
use synk_lib::commands::profile_card::get_profile_card_inner;

async fn seeded_pool() -> SqlitePool {
    let pool = crate::common::memory_pool().await;
    for i in 0..90u32 {
        let day =
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Days::new(u64::from(i));
        sqlx::query(
            "INSERT INTO body_measures_legacy (measured_at, weight, fat_ratio, source) \
             VALUES (?1, ?2, ?3, 'manual')",
        )
        .bind(format!("{}T07:00:00Z", day.format("%Y-%m-%d")))
        .bind(70.0 + f64::from(i % 10) / 10.0)
        .bind(21.0 + f64::from(i % 7) / 10.0)
        .execute(&pool)
        .await
        .unwrap();
    }
    sqlx::query(
        "INSERT INTO user_profile (id, age, sex) VALUES (1, 36, 'male') \
         ON CONFLICT(id) DO UPDATE SET age = 36, sex = 'male'",
    )
    .execute(&pool)
    .await
    .unwrap();
    for name in ["Lévothyrox 75 µg", "Vitamine D3"] {
        sqlx::query("INSERT INTO medications (name, active) VALUES (?1, 1)")
            .bind(name)
            .execute(&pool)
            .await
            .unwrap();
    }
    add_measurement_session_inner(
        &pool,
        "2026-03-15".into(),
        [
            ("chest", 92.0),
            ("waist", 86.0),
            ("hips", 98.5),
            ("neck", 39.0),
        ]
        .into_iter()
        .map(|(slug, value)| SessionEntry {
            slug: slug.into(),
            value,
        })
        .collect(),
    )
    .await
    .unwrap();
    pool
}

#[tokio::test]
#[ignore]
async fn dump_the_three_pdfs() {
    let dir = PathBuf::from(std::env::var("SYNK_PDF_DUMP").expect("SYNK_PDF_DUMP absent"));
    std::fs::create_dir_all(&dir).unwrap();
    let pool = seeded_pool().await;

    export_pdf_to(
        &pool,
        "2026-01-01",
        "2026-03-31",
        &["body_measures".to_string()],
        &dir.join("rapport.pdf"),
    )
    .await
    .unwrap();

    let rows = get_body_sheet_inner(&pool).await.unwrap();
    render_sheet(&rows, &dir.join("fiche.pdf")).unwrap();

    let card = get_profile_card_inner(&pool).await.unwrap();
    render_profile_card(&card, &dir.join("carte.pdf")).unwrap();
}
