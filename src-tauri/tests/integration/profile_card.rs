use crate::pdf_text;

use pdf_text::pdf_has_text;
use sqlx::SqlitePool;
use synk_lib::commands::profile_card::get_profile_card_inner;
use synk_lib::commands::profile_card::records::fetch_records;
use synk_lib::db::bootstrap_schema;

async fn insert_body(pool: &SqlitePool, date: &str, weight: f64) {
    sqlx::query(
        "INSERT INTO body_measures_legacy (measured_at, weight, source) VALUES (?1, ?2, 'manual')",
    )
    .bind(date)
    .bind(weight)
    .execute(pool)
    .await
    .unwrap();
}

// PDF non compresse (pas de feature FlateDecode ici) : le nombre de pages
// est verifiable directement dans l'objet catalog `/Type/Pages/Count N`,
// inspecte manuellement sur des PDF generes avant d'ecrire ce parseur.
fn pdf_page_count(bytes: &[u8]) -> Option<usize> {
    let text = String::from_utf8_lossy(bytes);
    let start = text.find("/Type/Pages")?;
    let after = &text[start..];
    let count_idx = after.find("/Count")?;
    let rest = after[count_idx + "/Count".len()..].trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

async fn column_exists(pool: &SqlitePool, column: &str) -> bool {
    let rows: Vec<(i64, String)> =
        sqlx::query_as("SELECT cid, name FROM pragma_table_info('user_profile')")
            .fetch_all(pool)
            .await
            .unwrap();
    rows.iter().any(|(_, name)| name == column)
}

#[tokio::test]
async fn migration_adds_the_nine_identity_columns() {
    let pool = crate::common::memory_pool().await;
    for column in [
        "birth_date",
        "handedness",
        "eye_color",
        "hair_color",
        "blood_group",
        "allergies",
        "vision_correction",
        "medical_history",
        "vaccines",
    ] {
        assert!(
            column_exists(&pool, column).await,
            "colonne absente : {column}"
        );
    }
}

#[tokio::test]
async fn legacy_columns_are_preserved() {
    let pool = crate::common::memory_pool().await;
    for column in ["age", "sex", "height_cm"] {
        assert!(
            column_exists(&pool, column).await,
            "colonne legacy perdue : {column}"
        );
    }
}

#[tokio::test]
async fn migration_is_idempotent() {
    let pool = crate::common::memory_pool().await;
    bootstrap_schema(&pool).await.unwrap();
    bootstrap_schema(&pool).await.unwrap();
    assert!(column_exists(&pool, "birth_date").await);
}

#[tokio::test]
async fn records_are_empty_on_fresh_database() {
    let pool = crate::common::memory_pool().await;
    let records = fetch_records(&pool).await.unwrap();
    assert!(records.is_empty(), "base vide : aucun record");
}

#[tokio::test]
async fn weight_records_carry_the_date_of_the_extreme() {
    let pool = crate::common::memory_pool().await;
    insert_body(&pool, "2026-01-15", 82.0).await;
    insert_body(&pool, "2026-04-20", 74.5).await;
    insert_body(&pool, "2026-07-10", 78.0).await;

    let records = fetch_records(&pool).await.unwrap();
    let min = records.iter().find(|r| r.key == "weight_min").unwrap();
    let max = records.iter().find(|r| r.key == "weight_max").unwrap();

    assert_eq!(min.value, 74.5);
    assert!(
        min.achieved_at.starts_with("2026-04-20"),
        "date du minimum : {}",
        min.achieved_at
    );
    assert_eq!(max.value, 82.0);
    assert!(
        max.achieved_at.starts_with("2026-01-15"),
        "date du maximum : {}",
        max.achieved_at
    );
}

#[tokio::test]
async fn steps_record_uses_the_best_day() {
    let pool = crate::common::memory_pool().await;
    for (date, steps) in [
        ("2026-02-01", 8000),
        ("2026-02-02", 15400),
        ("2026-02-03", 9100),
    ] {
        sqlx::query("INSERT INTO activity (date, steps) VALUES (?1, ?2)")
            .bind(date)
            .bind(steps)
            .execute(&pool)
            .await
            .unwrap();
    }
    let records = fetch_records(&pool).await.unwrap();
    let best = records.iter().find(|r| r.key == "steps_max").unwrap();
    assert_eq!(best.value, 15400.0);
    assert!(best.achieved_at.starts_with("2026-02-02"));
}

#[tokio::test]
async fn card_renders_on_empty_database() {
    let pool = crate::common::memory_pool().await;
    let card = get_profile_card_inner(&pool).await.unwrap();

    assert!(card.body.age.is_none());
    assert!(card.body.weight_kg.is_none());
    assert!(card.health.active_treatments.is_empty());
    assert!(card.records.is_empty());
    assert_eq!(
        card.key_measurements.len(),
        22,
        "les 22 mesures cles sont listees meme sans valeur"
    );
    assert!(card.key_measurements.iter().all(|m| m.value.is_none()));
}

#[tokio::test]
async fn card_reads_age_from_stored_profile() {
    let pool = crate::common::memory_pool().await;
    sqlx::query(
        "INSERT INTO user_profile (id, age) VALUES (1, 42) ON CONFLICT(id) DO UPDATE SET age = 42",
    )
    .execute(&pool)
    .await
    .unwrap();

    let card = get_profile_card_inner(&pool).await.unwrap();
    assert_eq!(card.body.age, Some(42));
}

#[tokio::test]
async fn card_lists_only_active_treatments() {
    let pool = crate::common::memory_pool().await;
    sqlx::query("INSERT INTO medications (name, active) VALUES ('Doliprane', 1)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO medications (name, active) VALUES ('Ancien traitement', 0)")
        .execute(&pool)
        .await
        .unwrap();

    let card = get_profile_card_inner(&pool).await.unwrap();
    assert_eq!(card.health.active_treatments, vec!["Doliprane".to_string()]);
}

#[tokio::test]
async fn card_reads_latest_body_measure() {
    let pool = crate::common::memory_pool().await;
    insert_body(&pool, "2026-01-10", 82.0).await;
    insert_body(&pool, "2026-07-30", 76.4).await;

    let card = get_profile_card_inner(&pool).await.unwrap();
    assert_eq!(card.body.weight_kg, Some(76.4));
    assert!(card
        .body
        .measured_at
        .as_deref()
        .unwrap()
        .starts_with("2026-07-30"));
}

#[tokio::test]
async fn generates_a_non_empty_profile_pdf() {
    use synk_lib::commands::export::render_profile_card;

    let pool = crate::common::memory_pool().await;
    sqlx::query(
        "INSERT INTO user_profile (id, age, sex) VALUES (1, 36, 'male') \
         ON CONFLICT(id) DO UPDATE SET age = 36, sex = 'male'",
    )
    .execute(&pool)
    .await
    .unwrap();
    insert_body(&pool, "2026-08-01", 74.5).await;

    let card = get_profile_card_inner(&pool).await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("carte.pdf");
    render_profile_card(&card, &path).unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert!(
        bytes.len() > 1000,
        "PDF suspicieusement petit : {}",
        bytes.len()
    );
    assert_eq!(&bytes[0..4], b"%PDF", "entete PDF absent");
}

#[tokio::test]
async fn profile_pdf_renders_on_empty_database() {
    use synk_lib::commands::export::render_profile_card;

    let pool = crate::common::memory_pool().await;
    let card = get_profile_card_inner(&pool).await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vide.pdf");
    render_profile_card(&card, &path).unwrap();
    assert!(std::fs::read(&path).unwrap().len() > 500);
}

#[tokio::test]
async fn profile_pdf_dense_content_produces_more_pages_than_an_empty_card() {
    use synk_lib::commands::export::render_profile_card;

    // Une carte vide liste deja les 22 mensurations cles : la comparaison
    // porte donc sur un STRICT plus grand nombre de pages, pas juste "> 1" :
    // sinon l'assertion serait vraie meme sans contenu dense.
    let empty_pool = crate::common::memory_pool().await;
    let empty_card = get_profile_card_inner(&empty_pool).await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let empty_path = dir.path().join("vide.pdf");
    render_profile_card(&empty_card, &empty_path).unwrap();
    let empty_pages = pdf_page_count(&std::fs::read(&empty_path).unwrap())
        .expect("marqueur /Type/Pages/Count introuvable dans le PDF vide");

    // Le seul bloc a contenu variable restant est "Traitements en cours" :
    // un grand nombre de medicaments actifs doit a lui seul faire deborder
    // sur une page supplementaire.
    let dense_pool = crate::common::memory_pool().await;
    for i in 0..60 {
        sqlx::query("INSERT INTO medications (name, active) VALUES (?1, 1)")
            .bind(format!("Traitement actif numero {i}"))
            .execute(&dense_pool)
            .await
            .unwrap();
    }

    let dense_card = get_profile_card_inner(&dense_pool).await.unwrap();
    let dense_path = dir.path().join("dense.pdf");
    render_profile_card(&dense_card, &dense_path).unwrap();
    let dense_pages = pdf_page_count(&std::fs::read(&dense_path).unwrap())
        .expect("marqueur /Type/Pages/Count introuvable dans le PDF dense");

    assert!(
        dense_pages > empty_pages,
        "le contenu dense doit produire strictement plus de pages qu'une carte vide : vide={empty_pages}, dense={dense_pages}"
    );
}

#[tokio::test]
async fn every_page_of_a_multi_page_card_carries_its_own_footer() {
    use synk_lib::commands::export::render_profile_card;

    // Meme recette que profile_pdf_dense_content_produces_more_pages_than_an_empty_card
    // pour forcer un debordement multi-pages fiable.
    let pool = crate::common::memory_pool().await;
    for i in 0..60 {
        sqlx::query("INSERT INTO medications (name, active) VALUES (?1, 1)")
            .bind(format!("Traitement actif numero {i}"))
            .execute(&pool)
            .await
            .unwrap();
    }
    let card = get_profile_card_inner(&pool).await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dense.pdf");
    render_profile_card(&card, &path).unwrap();

    let bytes = std::fs::read(&path).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    let page_count =
        pdf_page_count(&bytes).expect("marqueur /Type/Pages/Count introuvable dans le PDF dense");
    assert!(
        page_count >= 2,
        "le fixture doit produire au moins 2 pages pour que le test soit probant : {page_count}"
    );

    for page in 1..=page_count {
        let target = &format!("Synk - page {page} - généré le");
        assert!(
            pdf_has_text(&text, target),
            "pied de page absent sur la page {page} (total {page_count})"
        );
    }
    let phantom = &format!("Synk - page {}", page_count + 1);
    assert!(
        !pdf_has_text(&text, phantom),
        "pied de page inattendu au-dela des {page_count} pages reelles"
    );
}
