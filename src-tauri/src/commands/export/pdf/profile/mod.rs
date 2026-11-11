mod layout;
mod rows;

use crate::error::AppError;
use layout::section;
use rows::{body_rows, key_measurement_rows, record_rows, size_rows, treatment_rows};

use crate::commands::export::pdf::canvas::Canvas;
use crate::commands::export::pdf::layout::{stamp_footer, Cursor, LINE_MM, TOP_MM};
use printpdf::{BuiltinFont, Mm};
use sqlx::SqlitePool;
use std::path::Path;
use tauri::State;

use crate::commands::export::shared::resolve_output_dir;
use crate::commands::profile_card::get_profile_card_inner;
use crate::commands::profile_card::types::ProfileCard;

// `pub` requis : les tests d'integration dans `tests/` sont compiles comme
// des crates separes.
pub fn render_profile_card(card: &ProfileCard, output_path: &Path) -> Result<(), AppError> {
    let mut canvas = Canvas::new("Synk - Carte d'identité chiffrée");
    let font = BuiltinFont::Helvetica;
    let font_bold = BuiltinFont::HelveticaBold;

    let generated_on = chrono::Local::now().format("%d/%m/%Y").to_string();

    let mut cursor = Cursor { y: TOP_MM, page: 1 };
    stamp_footer(&mut canvas, font, cursor.page, &generated_on);

    canvas.use_text(
        "Carte d'identité chiffrée",
        18.0,
        Mm(20.0),
        Mm(cursor.y),
        font_bold,
    );
    cursor.y -= LINE_MM * 2.0;
    canvas.use_text(
        format!("Générée le {generated_on}"),
        9.0,
        Mm(20.0),
        Mm(cursor.y),
        font,
    );
    cursor.y -= LINE_MM * 2.0;

    // Le contenu deborde plutot que d'etre tronque : une carte medicale
    // amputee en silence est pire qu'un document a deux pages.
    section(
        &mut canvas,
        &mut cursor,
        font,
        font_bold,
        &generated_on,
        "Corps",
        body_rows(card),
    );

    section(
        &mut canvas,
        &mut cursor,
        font,
        font_bold,
        &generated_on,
        "Mensurations clés",
        key_measurement_rows(card),
    );

    section(
        &mut canvas,
        &mut cursor,
        font,
        font_bold,
        &generated_on,
        "Traitements en cours",
        treatment_rows(card),
    );

    section(
        &mut canvas,
        &mut cursor,
        font,
        font_bold,
        &generated_on,
        "Tailles",
        size_rows(card),
    );

    section(
        &mut canvas,
        &mut cursor,
        font,
        font_bold,
        &generated_on,
        "Records",
        record_rows(card),
    );

    canvas.save(output_path)
}

#[tauri::command]
pub async fn export_profile_card_pdf(
    app: tauri::AppHandle,
    pool: State<'_, SqlitePool>,
) -> Result<String, AppError> {
    let output_dir = resolve_output_dir(&app)?;
    tokio::fs::create_dir_all(&output_dir).await?;
    let filename = format!(
        "synk_profil_{}.pdf",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let output_path = output_dir.join(&filename);

    // Phase 1 : fetch async (sqlx, Send-friendly)
    let card = get_profile_card_inner(pool.inner()).await?;

    // Phase 2 : rendu sync
    let target = output_path.clone();
    tauri::async_runtime::spawn_blocking(move || render_profile_card(&card, &target))
        .await
        .map_err(|e| AppError::Internal(format!("rendu PDF interrompu : {e}")))??;

    Ok(output_path.to_string_lossy().into_owned())
}
