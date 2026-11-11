use super::canvas::Canvas;
use super::display::{fr_date, fr_number};
use super::figure_page::draw_figures;
use crate::error::AppError;
use printpdf::{BuiltinFont, Mm};
use sqlx::SqlitePool;
use std::path::Path;
use tauri::State;

use crate::body_catalog;
use crate::commands::body_profile::{get_body_sheet_inner, SheetRow};
use crate::commands::export::pdf::layout::{ensure_room, stamp_footer, Cursor, LINE_MM, TOP_MM};
use crate::commands::export::shared::resolve_output_dir;

pub fn dpi_for_width(px: u32, target_mm: f32) -> f32 {
    px as f32 * 25.4 / target_mm
}

pub(super) fn value_for<'a>(rows: &'a [SheetRow], slug: &str) -> Option<&'a SheetRow> {
    rows.iter().find(|r| r.slug == slug)
}

pub(super) fn format_value(row: Option<&SheetRow>) -> String {
    match row.and_then(|r| r.value) {
        Some(v) => fr_number(v, 1),
        None => "-".to_string(),
    }
}

pub struct TableLine {
    pub is_zone_header: bool,
    pub label: String,
    pub value: String,
    pub unit: String,
    pub measured_at: String,
}

pub fn sheet_table_lines(rows: &[SheetRow]) -> Vec<TableLine> {
    let zones = [
        ("global", "Global"),
        ("neck_shoulders", "Cou et épaules"),
        ("torso", "Tronc"),
        ("arms", "Bras"),
        ("legs", "Jambes"),
    ];
    let mut lines = Vec::new();
    for (zone_id, zone_label) in zones {
        lines.push(TableLine {
            is_zone_header: true,
            label: zone_label.to_string(),
            value: String::new(),
            unit: String::new(),
            measured_at: String::new(),
        });
        let mut defs: Vec<_> = body_catalog::catalog()
            .iter()
            .filter(|m| m.zone == zone_id)
            .collect();
        defs.sort_by_key(|m| m.order);
        for m in defs {
            let row = value_for(rows, &m.slug);
            lines.push(TableLine {
                is_zone_header: false,
                label: m.label.clone(),
                value: format_value(row),
                unit: "cm".to_string(),
                measured_at: row
                    .and_then(|r| r.measured_at.as_deref())
                    .map(fr_date)
                    .unwrap_or_default(),
            });
        }
    }
    lines
}

pub fn render_sheet(rows: &[SheetRow], output_path: &Path) -> Result<(), AppError> {
    let mut canvas = Canvas::new("Synk - Fiche de mensurations");
    let font = BuiltinFont::Helvetica;
    let font_bold = BuiltinFont::HelveticaBold;

    let generated_on = chrono::Local::now().format("%d/%m/%Y").to_string();

    stamp_footer(&mut canvas, font, 1, &generated_on);

    let left = draw_figures(&mut canvas, rows)?;
    canvas.use_text(
        "Fiche de mensurations",
        18.0,
        Mm(left),
        Mm(272.0),
        font_bold,
    );
    canvas.use_text(
        format!("Générée le {generated_on}"),
        9.0,
        Mm(left),
        Mm(265.0),
        font,
    );

    canvas.add_page();
    let mut cursor = Cursor { y: TOP_MM, page: 2 };
    stamp_footer(&mut canvas, font, cursor.page, &generated_on);

    canvas.use_text(
        "Detail des mensurations",
        14.0,
        Mm(20.0),
        Mm(cursor.y),
        font_bold,
    );
    cursor.y -= LINE_MM * 2.0;

    for line in sheet_table_lines(rows) {
        if line.is_zone_header {
            ensure_room(&mut canvas, &mut cursor, font, &generated_on, LINE_MM * 2.0);
            canvas.use_text(&line.label, 10.0, Mm(20.0), Mm(cursor.y), font_bold);
            cursor.y -= LINE_MM * 1.2;
            continue;
        }
        ensure_room(&mut canvas, &mut cursor, font, &generated_on, LINE_MM);
        canvas.use_text(&line.label, 8.0, Mm(24.0), Mm(cursor.y), font);
        canvas.use_text(&line.value, 8.0, Mm(120.0), Mm(cursor.y), font);
        canvas.use_text(&line.unit, 8.0, Mm(134.0), Mm(cursor.y), font);
        canvas.use_text(&line.measured_at, 8.0, Mm(148.0), Mm(cursor.y), font);
        cursor.y -= LINE_MM;
    }

    canvas.save(output_path)
}

#[tauri::command]
pub async fn export_body_sheet_pdf(
    app: tauri::AppHandle,
    pool: State<'_, SqlitePool>,
) -> Result<String, AppError> {
    let output_dir = resolve_output_dir(&app)?;
    tokio::fs::create_dir_all(&output_dir).await?;
    let filename = format!(
        "synk_mensurations_{}.pdf",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let output_path = output_dir.join(&filename);

    let rows = get_body_sheet_inner(pool.inner()).await?;

    let target = output_path.clone();
    tauri::async_runtime::spawn_blocking(move || render_sheet(&rows, &target))
        .await
        .map_err(|e| AppError::Internal(format!("rendu PDF interrompu : {e}")))??;

    Ok(output_path.to_string_lossy().into_owned())
}
