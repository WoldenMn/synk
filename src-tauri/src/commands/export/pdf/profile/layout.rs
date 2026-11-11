use printpdf::{BuiltinFont, Mm};

use crate::commands::export::pdf::canvas::Canvas;

use crate::commands::export::pdf::layout::{ensure_room, wrap_text, Cursor, LINE_MM};

const VALUE_X_MM: f32 = 95.0;
// Colonne valeur ouverte de 95mm a ~195mm (marge droite 15mm sur une page
// A4 de 210mm) : 100mm disponibles. Verifie par calcul sur les metriques
// officielles Adobe Core14 AFM de Helvetica (WX en 1/1000 em, table
// recuperee sur github.com/tecnickcom/tc-font-core14-afms), pas une
// estimation a vue de nez ni un rendu visuel (indisponible dans cet
// environnement) : a 9pt, une phrase francaise realiste de 54 caracteres
// mesure ~81mm, soit une marge de securite d'environ 19mm avant le bord de
// colonne pour absorber un vocabulaire plus large que l'echantillon teste.
const VALUE_MAX_CHARS: usize = 54;

pub(super) fn section(
    canvas: &mut Canvas,
    cursor: &mut Cursor,
    font: BuiltinFont,
    font_bold: BuiltinFont,
    generated_on: &str,
    title: &str,
    rows: Vec<(String, String)>,
) {
    ensure_room(canvas, cursor, font, generated_on, LINE_MM * 2.0);
    canvas.use_text(title, 12.0, Mm(20.0), Mm(cursor.y), font_bold);
    cursor.y -= LINE_MM * 1.4;
    for (label, value) in rows {
        // Une valeur longue deborde sur autant de lignes que necessaire,
        // chacune passant par ensure_room : jamais de texte coupe hors
        // cadre, jamais de page ajoutee au milieu d'une ligne deja tracee.
        let mut lines = wrap_text(&value, VALUE_MAX_CHARS);
        if lines.is_empty() {
            lines.push(String::new());
        }
        let mut lines = lines.into_iter();

        ensure_room(canvas, cursor, font, generated_on, LINE_MM);
        canvas.use_text(&label, 9.0, Mm(24.0), Mm(cursor.y), font);
        canvas.use_text(
            lines.next().unwrap(),
            9.0,
            Mm(VALUE_X_MM),
            Mm(cursor.y),
            font,
        );
        cursor.y -= LINE_MM;

        for extra in lines {
            ensure_room(canvas, cursor, font, generated_on, LINE_MM);
            canvas.use_text(extra, 9.0, Mm(VALUE_X_MM), Mm(cursor.y), font);
            cursor.y -= LINE_MM;
        }
    }
    cursor.y -= LINE_MM * 0.6;
}
