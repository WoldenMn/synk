use printpdf::{BuiltinFont, ParsedFont};

use crate::error::AppError;

const MM_PER_PT: f32 = 25.4 / 72.0;

pub(crate) struct TextMetrics {
    regular: ParsedFont,
    bold: ParsedFont,
}

impl TextMetrics {
    pub(crate) fn helvetica() -> Result<Self, AppError> {
        let load = |font: BuiltinFont| {
            font.get_parsed_font()
                .ok_or_else(|| AppError::Internal(format!("police {font:?} illisible")))
        };
        Ok(Self {
            regular: load(BuiltinFont::Helvetica)?,
            bold: load(BuiltinFont::HelveticaBold)?,
        })
    }

    pub(crate) fn width_mm(&self, text: &str, size_pt: f32, bold: bool) -> f32 {
        let font = if bold { &self.bold } else { &self.regular };
        let em = f32::from(font.units_per_em.max(1));
        let units: f32 = text
            .chars()
            .map(|c| {
                font.lookup_glyph_index(u32::from(c))
                    .and_then(|gid| font.get_glyph_width(gid))
                    .map_or(em, f32::from)
            })
            .sum();
        units / em * size_pt * MM_PER_PT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> TextMetrics {
        TextMetrics::helvetica().unwrap()
    }

    #[test]
    fn measures_the_advances_of_helvetica() {
        let m = metrics();
        let em = 10.0 * MM_PER_PT;
        assert!((m.width_mm("a", 10.0, false) - 0.556 * em).abs() < 0.01);
        assert!((m.width_mm("i", 10.0, false) - 0.222 * em).abs() < 0.01);
        assert!((m.width_mm("m", 10.0, false) - 0.833 * em).abs() < 0.01);
    }

    #[test]
    fn gives_an_accented_letter_the_width_of_its_base_letter() {
        let m = metrics();
        assert_eq!(m.width_mm("é", 7.0, false), m.width_mm("e", 7.0, false));
        assert_eq!(m.width_mm("é", 7.0, true), m.width_mm("e", 7.0, true));
    }

    #[test]
    fn grows_with_the_text_and_the_size() {
        let m = metrics();
        let short = m.width_mm("Tour de cou", 7.0, false);
        let long = m.width_mm("Longueur de bras tendu G/D", 7.0, false);
        assert!(long > short * 2.0, "{long} contre {short}");
        let doubled = m.width_mm("Tour de cou", 14.0, false);
        assert!((doubled - short * 2.0).abs() < 1e-4);
        assert!(m.width_mm("Tour de cou", 7.0, true) > short);
    }

    #[test]
    fn counts_an_unknown_character_as_a_full_em() {
        let m = metrics();
        let em = 7.0 * MM_PER_PT;
        assert!((m.width_mm("\u{10FFFD}", 7.0, false) - em).abs() < 1e-4);
    }
}
