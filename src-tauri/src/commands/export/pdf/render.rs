use crate::error::AppError;
use printpdf::{BuiltinFont, Mm};
use std::path::Path;

use super::canvas::Canvas;
use super::display::fr_date;
use super::fetch::PdfData;
use super::layout::{ensure_room, stamp_footer, Cursor, LINE_MM, TOP_MM};
use super::report_lines::report_sections;

const LEFT_MM: f32 = 20.0;

// Le rapport s'etend sur autant de pages que ses donnees : chaque ligne passe
// par ensure_room, qui ouvre une page et y pose le pied de page. L'ancien rendu
// s'arretait en bas de la premiere page (`if y < 40.0 { break }`) et perdait les
// lignes suivantes sans le dire.
pub(super) fn render_pdf(
    data: &PdfData,
    output_path: &Path,
    date_from: &str,
    date_to: &str,
    categories: &[String],
) -> Result<(), AppError> {
    let generated_on = chrono::Local::now().format("%d/%m/%Y").to_string();
    let mut canvas = Canvas::new("Synk - Rapport de santé");
    let font = BuiltinFont::Helvetica;
    let bold = BuiltinFont::HelveticaBold;

    let mut cursor = Cursor { y: TOP_MM, page: 1 };
    stamp_footer(&mut canvas, font, cursor.page, &generated_on);

    canvas.use_text(
        "Synk - Rapport de santé",
        16.0,
        Mm(LEFT_MM),
        Mm(cursor.y),
        bold,
    );
    cursor.y -= LINE_MM * 2.0;
    canvas.use_text(
        format!("Période du {} au {}", fr_date(date_from), fr_date(date_to)),
        10.0,
        Mm(LEFT_MM),
        Mm(cursor.y),
        font,
    );
    cursor.y -= LINE_MM * 2.5;

    for section in report_sections(data, categories) {
        // Le titre ne reste jamais seul en bas de page : il part avec sa
        // premiere ligne.
        ensure_room(&mut canvas, &mut cursor, font, &generated_on, LINE_MM * 3.0);
        canvas.use_text(section.title, 12.0, Mm(LEFT_MM), Mm(cursor.y), bold);
        cursor.y -= LINE_MM * 1.5;

        let lines = if section.lines.is_empty() {
            vec!["Aucune donnée sur la période".to_string()]
        } else {
            section.lines
        };
        for line in lines {
            ensure_room(&mut canvas, &mut cursor, font, &generated_on, LINE_MM);
            canvas.use_text(line, 9.0, Mm(LEFT_MM + 4.0), Mm(cursor.y), font);
            cursor.y -= LINE_MM;
        }
        cursor.y -= LINE_MM * 0.6;
    }

    canvas.save(output_path)
}
