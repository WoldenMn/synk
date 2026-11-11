use crate::pdf_text;

use pdf_text::pdf_has_text;
use sqlx::SqlitePool;
use synk_lib::commands::export::export_pdf_to;

// PDF non compresse : le nombre de pages se lit dans `/Type/Pages/Count N`
// (meme lecture que tests/profile_card.rs).
fn pdf_page_count(bytes: &[u8]) -> Option<usize> {
    let text = String::from_utf8_lossy(bytes);
    let start = text.find("/Type/Pages")?;
    let after = &text[start..];
    let count_idx = after.find("/Count")?;
    let rest = after[count_idx + "/Count".len()..].trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

async fn seed_weighings(pool: &SqlitePool, count: u32) {
    for i in 0..count {
        let day =
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Days::new(i as u64);
        sqlx::query(
            "INSERT INTO body_measures_legacy (measured_at, weight, fat_ratio, source) \
             VALUES (?1, ?2, 21.647000000000002, 'manual')",
        )
        .bind(format!("{}T07:00:00Z", day.format("%Y-%m-%d")))
        .bind(70.0 + f64::from(i % 10) / 10.0)
        .execute(pool)
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn a_long_report_spans_several_pages_and_keeps_its_last_row() {
    let pool = crate::common::memory_pool().await;
    seed_weighings(&pool, 120).await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rapport.pdf");

    export_pdf_to(
        &pool,
        "2026-01-01",
        "2026-06-30",
        &["body_measures".to_string()],
        &path,
    )
    .await
    .unwrap();

    let bytes = std::fs::read(&path).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    let pages = pdf_page_count(&bytes).expect("compte de pages introuvable");
    assert!(
        pages >= 3,
        "120 pesees tiennent sur au moins 3 pages : {pages}"
    );
    for page in 1..=pages {
        assert!(
            pdf_has_text(&text, &format!("Synk - page {page} - généré le")),
            "pied de page absent sur la page {page}"
        );
    }
    // Premiere et derniere pesee (01/01 et 30/04/2026) : rien n'est perdu en bas de page.
    assert!(pdf_has_text(&text, "01/01/2026   Poids 70,0 kg"));
    assert!(
        pdf_has_text(&text, "30/04/2026   Poids"),
        "la derniere pesee manque : le rapport a ete tronque"
    );
}

#[tokio::test]
async fn report_values_are_written_the_french_way() {
    let pool = crate::common::memory_pool().await;
    seed_weighings(&pool, 1).await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rapport.pdf");

    export_pdf_to(
        &pool,
        "2026-01-01",
        "2026-01-31",
        &["body_measures".to_string(), "sleep".to_string()],
        &path,
    )
    .await
    .unwrap();

    let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
    assert!(pdf_has_text(&text, "Période du 01/01/2026 au 31/01/2026"));
    assert!(pdf_has_text(&text, "Graisse 21,6 %"));
    assert!(!pdf_has_text(&text, "21.647"), "valeur brute non arrondie");
    assert!(pdf_has_text(&text, "Aucune donnée sur la période"));
}

#[tokio::test]
async fn an_empty_period_writes_no_pdf() {
    let dir = tempfile::tempdir().unwrap();
    let pool = synk_lib::db::init_pool(dir.path().join("synk.db"))
        .await
        .unwrap();
    let out = dir.path().join("vide.pdf");
    let rows = export_pdf_to(
        &pool,
        "2026-01-01T00:00:00.000Z",
        "2026-01-31T23:59:59.000Z",
        &["body_measures".to_string()],
        &out,
    )
    .await
    .unwrap();
    assert_eq!(rows, 0);
    assert!(!out.exists());
}
