use printpdf::{BuiltinFont, Mm};

use super::canvas::Canvas;

pub(crate) const PAGE_WIDTH_MM: f32 = 210.0;
pub(crate) const PAGE_HEIGHT_MM: f32 = 297.0;
pub(crate) const TOP_MM: f32 = 272.0;
pub(crate) const BOTTOM_MM: f32 = 20.0;
pub(crate) const LINE_MM: f32 = 5.0;

pub(crate) struct Cursor {
    pub(crate) y: f32,
    pub(crate) page: usize,
}

// Coupe aux espaces, jamais au milieu d'un mot. Un mot isolement plus long
// que `max_chars` (URL, reference sans espace) est coupe brutalement pour
// eviter de bloquer le retour a la ligne. Chaine vide -> vecteur vide.
pub(crate) fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    if text.is_empty() || max_chars == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if word.chars().count() > max_chars {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
            }
            let mut chunk = String::new();
            for ch in word.chars() {
                chunk.push(ch);
                if chunk.chars().count() == max_chars {
                    lines.push(std::mem::take(&mut chunk));
                }
            }
            if !chunk.is_empty() {
                current = chunk;
            }
            continue;
        }
        let candidate_len = if current.is_empty() {
            word.chars().count()
        } else {
            current.chars().count() + 1 + word.chars().count()
        };
        if candidate_len > max_chars {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

// Numerotation sans total : printpdf ne connait le nombre final de pages
// qu'apres coup, et retraverser le document pour le stamper couterait plus
// que ce que ca apporte. « Page 3 » suffit a savoir qu'il en manque une.
pub(crate) fn stamp_footer(
    canvas: &mut Canvas,
    font: BuiltinFont,
    page: usize,
    generated_on: &str,
) {
    canvas.use_text(
        format!("Synk - page {page} - généré le {generated_on}"),
        7.0,
        Mm(20.0),
        Mm(12.0),
        font,
    );
}

pub(crate) fn ensure_room(
    canvas: &mut Canvas,
    cursor: &mut Cursor,
    font: BuiltinFont,
    generated_on: &str,
    needed: f32,
) {
    if cursor.y - needed < BOTTOM_MM {
        canvas.add_page();
        cursor.y = TOP_MM;
        cursor.page += 1;
        stamp_footer(canvas, font, cursor.page, generated_on);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_at_word_boundaries_never_splitting_a_word() {
        let lines = wrap_text("aaaa bbbb cccc", 9);
        assert_eq!(lines, vec!["aaaa bbbb".to_string(), "cccc".to_string()]);
    }

    #[test]
    fn splits_a_single_word_longer_than_the_limit() {
        let lines = wrap_text("abcdefghij", 4);
        assert_eq!(
            lines,
            vec!["abcd".to_string(), "efgh".to_string(), "ij".to_string()]
        );
    }

    #[test]
    fn empty_string_returns_empty_vec_not_a_vec_of_one_empty_string() {
        assert!(wrap_text("", 10).is_empty());
    }
}
