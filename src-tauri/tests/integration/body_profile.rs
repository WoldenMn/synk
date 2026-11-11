use crate::pdf_text;

use pdf_text::pdf_has_text;
use sqlx::SqlitePool;
use synk_lib::body_catalog::catalog;
use synk_lib::commands::body_profile::{
    add_measurement_session_inner, get_body_sheet_inner, SessionEntry,
};
use synk_lib::commands::export::dpi_for_width;
use synk_lib::db::bootstrap_schema;

async fn count_measurements(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM manual_measurements_legacy")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn seeds_every_catalog_slug() {
    let pool = crate::common::memory_pool().await;
    for m in catalog() {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM measurement_types_legacy WHERE slug = ?1 AND is_builtin = 1",
        )
        .bind(&m.slug)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 1, "slug absent ou duplique en base : {}", m.slug);
    }
}

#[tokio::test]
async fn seeding_is_idempotent() {
    let pool = crate::common::memory_pool().await;
    bootstrap_schema(&pool).await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM measurement_types_legacy")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count as usize, catalog().len());
}

#[tokio::test]
async fn no_builtin_outside_catalog() {
    let pool = crate::common::memory_pool().await;
    let slugs: Vec<String> =
        sqlx::query_scalar("SELECT slug FROM measurement_types_legacy WHERE is_builtin = 1")
            .fetch_all(&pool)
            .await
            .unwrap();
    for slug in slugs {
        assert!(
            catalog().iter().any(|m| m.slug == slug),
            "built-in en base absent du catalogue : {slug}"
        );
    }
}

#[tokio::test]
async fn inseam_category_is_length() {
    let pool = crate::common::memory_pool().await;
    let category: String =
        sqlx::query_scalar("SELECT category FROM measurement_types_legacy WHERE slug = 'inseam'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(category, "length");
}

#[tokio::test]
async fn seeding_does_not_overwrite_user_custom_type_on_slug_collision() {
    let pool = crate::common::memory_pool().await;

    sqlx::query(
        "UPDATE measurement_types_legacy SET label = 'Ma mesure perso', is_builtin = 0 \
         WHERE slug = 'height'",
    )
    .execute(&pool)
    .await
    .unwrap();

    bootstrap_schema(&pool).await.unwrap();

    let row: (String, i64) = sqlx::query_as(
        "SELECT label, is_builtin FROM measurement_types_legacy WHERE slug = 'height'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        row.0, "Ma mesure perso",
        "le label utilisateur ne doit pas etre ecrase par le seeding"
    );
    assert_eq!(
        row.1, 0,
        "un type personnalise ne doit pas etre convertit en builtin par le seeding"
    );
}

#[tokio::test]
async fn writes_every_entry_of_a_session() {
    let pool = crate::common::memory_pool().await;
    let entries = vec![
        SessionEntry {
            slug: "chest".into(),
            value: 92.0,
        },
        SessionEntry {
            slug: "waist".into(),
            value: 86.0,
        },
        SessionEntry {
            slug: "hips".into(),
            value: 100.0,
        },
    ];
    let written = add_measurement_session_inner(&pool, "2026-07-31".into(), entries)
        .await
        .unwrap();
    assert_eq!(written, 3);
    assert_eq!(count_measurements(&pool).await, 3);
}

#[tokio::test]
async fn rejects_out_of_bounds_and_writes_nothing() {
    let pool = crate::common::memory_pool().await;
    let entries = vec![
        SessionEntry {
            slug: "chest".into(),
            value: 92.0,
        },
        SessionEntry {
            slug: "wrist_left".into(),
            value: 300.0,
        },
    ];
    let err = add_measurement_session_inner(&pool, "2026-07-31".into(), entries)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("poignet"), "message: {err}");
    assert_eq!(
        count_measurements(&pool).await,
        0,
        "rien ne doit etre ecrit"
    );
}

#[tokio::test]
async fn rejects_unknown_slug() {
    let pool = crate::common::memory_pool().await;
    let entries = vec![SessionEntry {
        slug: "inconnu".into(),
        value: 10.0,
    }];
    assert!(
        add_measurement_session_inner(&pool, "2026-07-31".into(), entries)
            .await
            .is_err()
    );
    assert_eq!(count_measurements(&pool).await, 0);
}

#[tokio::test]
async fn rejects_non_finite_value() {
    let pool = crate::common::memory_pool().await;
    let entries = vec![SessionEntry {
        slug: "chest".into(),
        value: f64::NAN,
    }];
    assert!(
        add_measurement_session_inner(&pool, "2026-07-31".into(), entries)
            .await
            .is_err()
    );
    assert_eq!(count_measurements(&pool).await, 0);
}

#[tokio::test]
async fn rejects_empty_session() {
    let pool = crate::common::memory_pool().await;
    assert!(
        add_measurement_session_inner(&pool, "2026-07-31".into(), vec![])
            .await
            .is_err()
    );
}

#[tokio::test]
async fn sheet_lists_every_catalog_measurement() {
    let pool = crate::common::memory_pool().await;
    let rows = get_body_sheet_inner(&pool).await.unwrap();
    assert_eq!(rows.len(), catalog().len());
    assert!(
        rows.iter().all(|r| r.value.is_none()),
        "base vide : aucune valeur"
    );
}

#[tokio::test]
async fn sheet_returns_latest_value_per_slug() {
    let pool = crate::common::memory_pool().await;
    add_measurement_session_inner(
        &pool,
        "2026-07-01".into(),
        vec![SessionEntry {
            slug: "waist".into(),
            value: 88.0,
        }],
    )
    .await
    .unwrap();
    add_measurement_session_inner(
        &pool,
        "2026-07-31".into(),
        vec![SessionEntry {
            slug: "waist".into(),
            value: 86.0,
        }],
    )
    .await
    .unwrap();

    let rows = get_body_sheet_inner(&pool).await.unwrap();
    let waist = rows.iter().find(|r| r.slug == "waist").unwrap();
    assert_eq!(waist.value, Some(86.0));
    assert_eq!(waist.measured_at.as_deref(), Some("2026-07-31"));
}

#[tokio::test]
async fn sheet_is_ordered_by_catalog_order() {
    let pool = crate::common::memory_pool().await;
    let rows = get_body_sheet_inner(&pool).await.unwrap();
    let expected: Vec<&str> = {
        let mut c: Vec<_> = catalog().iter().collect();
        c.sort_by_key(|m| m.order);
        c.iter().map(|m| m.slug.as_str()).collect()
    };
    let actual: Vec<&str> = rows.iter().map(|r| r.slug.as_str()).collect();
    assert_eq!(actual, expected);
}

#[test]
fn dpi_matches_target_width() {
    // 850 px rendus sur 80 mm : 80 mm = 3.1496 pouces -> ~269.9 dpi
    let dpi = dpi_for_width(850, 80.0);
    assert!((dpi - 269.9).abs() < 0.5, "dpi calcule : {dpi}");
}

#[test]
fn dpi_scales_with_target() {
    let wide = dpi_for_width(850, 160.0);
    let narrow = dpi_for_width(850, 80.0);
    assert!((narrow - wide * 2.0).abs() < 0.5);
}

#[tokio::test]
async fn every_measurement_appears_in_the_sheet_table() {
    use synk_lib::commands::export::sheet_table_lines;

    let pool = crate::common::memory_pool().await;
    let rows = get_body_sheet_inner(&pool).await.unwrap();
    let lines = sheet_table_lines(&rows);

    let measurements = lines.iter().filter(|l| !l.is_zone_header).count();
    assert_eq!(
        measurements,
        synk_lib::body_catalog::catalog().len(),
        "les 47 mensurations doivent etre presentes dans le tableau"
    );

    let headers = lines.iter().filter(|l| l.is_zone_header).count();
    assert_eq!(headers, 5, "5 en-tetes de zone");
}

#[tokio::test]
async fn the_sheet_pdf_spills_onto_a_third_page() {
    use synk_lib::commands::export::render_sheet;

    let pool = crate::common::memory_pool().await;
    let rows = get_body_sheet_inner(&pool).await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fiche.pdf");
    render_sheet(&rows, &path).unwrap();

    let bytes = std::fs::read(&path).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    // 47 mensurations et 5 en-tetes ne tiennent pas sur une seule page de
    // tableau : le document doit deborder au lieu de couper.
    assert!(
        text.contains("/Count 3"),
        "le tableau doit deborder sur une troisieme page"
    );
}

#[tokio::test]
async fn every_page_of_the_sheet_pdf_carries_its_own_footer() {
    use synk_lib::commands::export::render_sheet;

    let pool = crate::common::memory_pool().await;
    let rows = get_body_sheet_inner(&pool).await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fiche.pdf");
    render_sheet(&rows, &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let text = String::from_utf8_lossy(&bytes);

    // 3 pages reelles (cf. the_sheet_pdf_spills_onto_a_third_page) : chacune
    // doit porter son propre numero, et aucune page fantome au-dela.
    for page in 1..=3 {
        let target = &format!("Synk - page {page} - généré le");
        assert!(
            pdf_has_text(&text, target),
            "pied de page absent sur la page {page}"
        );
    }
    let phantom = "Synk - page 4";
    assert!(
        !pdf_has_text(&text, phantom),
        "pied de page inattendu au-dela des 3 pages reelles"
    );
}

#[tokio::test]
async fn generates_a_non_empty_pdf() {
    use synk_lib::commands::export::render_sheet;

    let pool = crate::common::memory_pool().await;
    add_measurement_session_inner(
        &pool,
        "2026-07-31".into(),
        vec![
            SessionEntry {
                slug: "chest".into(),
                value: 92.0,
            },
            SessionEntry {
                slug: "waist".into(),
                value: 86.0,
            },
        ],
    )
    .await
    .unwrap();

    let rows = get_body_sheet_inner(&pool).await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fiche.pdf");
    render_sheet(&rows, &path).unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert!(
        bytes.len() > 1000,
        "PDF suspicieusement petit : {}",
        bytes.len()
    );
    assert_eq!(&bytes[0..4], b"%PDF", "entete PDF absent");
    let content = String::from_utf8_lossy(&bytes);
    // Le tableau liste toujours les 47 mensurations du catalogue, quel que
    // soit le nombre de valeurs saisies : il deborde donc systematiquement
    // sur une troisieme page (cf. the_sheet_pdf_spills_onto_a_third_page).
    assert!(content.contains("/Count 3"), "le PDF doit contenir 3 pages");
    assert_eq!(
        pdf_text::page_one_xobject_draws(&bytes),
        2,
        "les 2 figures (front/back) doivent etre posees sur la page 1"
    );
    assert!(pdf_has_text(&content, "Detail des mensurations"));
    assert!(!pdf_has_text(&content, "Detail des mesures"));
}

fn png_size(bytes: &[u8]) -> (i64, i64) {
    let read = |at: usize| i64::from(u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()));
    (read(16), read(20))
}

#[tokio::test]
async fn the_sheet_pdf_carries_both_figures_unaltered() {
    use synk_lib::commands::export::render_sheet;

    let pool = crate::common::memory_pool().await;
    let rows = get_body_sheet_inner(&pool).await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fiche.pdf");
    render_sheet(&rows, &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();

    let sources = [
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../src/assets/body/front.png"
        ))
        .as_slice(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../src/assets/body/back.png"
        ))
        .as_slice(),
    ];
    let (width, height) = png_size(sources[0]);
    assert_eq!(png_size(sources[1]), (width, height));

    let images = pdf_text::page_one_images(&bytes);
    assert_eq!(images.len(), 2);
    for image in &images {
        assert_eq!(
            (image.width, image.height),
            (width, height),
            "la figure doit partir a sa taille d'origine, sans reduction"
        );
        assert!(
            image.has_soft_mask,
            "la transparence de la figure doit survivre"
        );
        assert_eq!(image.color_space.as_deref(), Some("DeviceGray"));
    }

    let draws = pdf_text::page_one_draw_matrices(&bytes);
    assert_eq!(draws.len(), 2);
    assert!(
        (draws[0][0] - draws[1][0]).abs() < 1e-3,
        "les deux vues doivent partager la meme echelle"
    );
    for m in draws {
        let width_mm = m[0] * 25.4 / 72.0;
        assert!(
            (45.0..=70.0).contains(&width_mm),
            "largeur de la figure sur la page : {width_mm} mm"
        );
        let page_aspect = f64::from(m[3] / m[0]);
        let png_aspect = height as f64 / width as f64;
        assert!(
            (page_aspect - png_aspect).abs() < 1e-3,
            "figure deformee : {page_aspect} contre {png_aspect}"
        );
    }
}
