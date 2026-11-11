use crate::error::AppError;

pub const MAX_BYTES: u64 = 256 * 1024 * 1024;

pub fn decode_text(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => bytes.iter().map(|b| char::from(*b)).collect(),
    }
}

pub fn detect_delimiter(text: &str) -> char {
    let lines: Vec<&str> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(12)
        .collect();
    [';', '\t', ',']
        .into_iter()
        .max_by_key(|d| {
            lines
                .iter()
                .map(|l| l.matches(*d).count())
                .min()
                .unwrap_or(0)
        })
        .unwrap_or(',')
}

pub fn rows(text: &str, delimiter: char) -> Result<Vec<Vec<String>>, AppError> {
    let byte = u8::try_from(delimiter)
        .map_err(|_| AppError::Validation("séparateur de colonnes invalide".into()))?;
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(byte)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let mut all = Vec::new();
    for record in reader.records() {
        let record =
            record.map_err(|e| AppError::Validation(format!("fichier CSV illisible : {e}")))?;
        let row: Vec<String> = record.iter().map(|c| c.trim().to_string()).collect();
        if row.iter().any(|c| !c.is_empty()) {
            all.push(row);
        }
    }
    Ok(all)
}

pub fn header_row(rows: &[Vec<String>]) -> usize {
    let window = &rows[..rows.len().min(10)];
    let widest = window.iter().map(Vec::len).max().unwrap_or(0);
    let full = |r: &Vec<String>| r.len() == widest && r.iter().all(|c| !c.is_empty());
    window
        .iter()
        .position(full)
        .or_else(|| window.iter().position(|r| r.len() == widest))
        .unwrap_or(0)
}

pub fn parse_number(text: &str, decimal: char) -> Option<f64> {
    let cleaned: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{a0}' && *c != '\u{202f}')
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    let normalised = if decimal == ',' {
        cleaned.replace('.', "").replace(',', ".")
    } else {
        cleaned.replace(',', "")
    };
    normalised.parse::<f64>().ok().filter(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_delimiter_is_the_one_every_line_carries() {
        assert_eq!(detect_delimiter("a;b;c\n1,5;2;3\n"), ';');
        assert_eq!(detect_delimiter("a,b,c\n1.5,2,3\n"), ',');
        assert_eq!(detect_delimiter("a\tb\n1\t2\n"), '\t');
    }

    #[test]
    fn quoted_cells_and_blank_lines_are_handled() {
        let r = rows("Date;Note\n\n25/10;\"a;b\"\n", ';').unwrap();
        assert_eq!(r, vec![vec!["Date", "Note"], vec!["25/10", "a;b"]]);
    }

    #[test]
    fn a_byte_order_mark_and_a_windows_text_are_decoded() {
        assert_eq!(decode_text(b"\xEF\xBB\xBFPoids"), "Poids");
        assert_eq!(decode_text(b"Temp\xE9rature"), "Température");
    }

    #[test]
    fn the_header_is_the_first_widest_row() {
        let r = rows("Rapport patient,,\nA,B,C\n1,2,3\n", ',').unwrap();
        assert_eq!(header_row(&r), 1);
        let r = rows("Rapport,x,y\n,,\nA,B,C\n1,2,3\n", ',').unwrap();
        assert_eq!(header_row(&r), 0);
    }

    #[test]
    fn numbers_follow_the_chosen_decimal_mark() {
        assert_eq!(parse_number("72,4", ','), Some(72.4));
        assert_eq!(parse_number("1 234,5", ','), Some(1234.5));
        assert_eq!(parse_number("72.4", '.'), Some(72.4));
        assert_eq!(parse_number("1,234.5", '.'), Some(1234.5));
        assert_eq!(parse_number("", '.'), None);
        assert_eq!(parse_number("abc", '.'), None);
    }
}
