const DATE_UNITS: &[(char, f64)] = &[('D', 86_400.0)];
const TIME_UNITS: &[(char, f64)] = &[('H', 3_600.0), ('M', 60.0), ('S', 1.0)];

fn part_seconds(part: &str, units: &[(char, f64)]) -> Option<f64> {
    let mut total = 0.0;
    let mut number = String::new();
    for ch in part.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            number.push(ch);
            continue;
        }
        let factor = units.iter().find(|(unit, _)| *unit == ch)?.1;
        total += number.parse::<f64>().ok()? * factor;
        number.clear();
    }
    number.is_empty().then_some(total)
}

pub fn iso_seconds(text: &str) -> Option<f64> {
    let body = text.strip_prefix('P')?;
    let (date_part, time_part) = body.split_once('T').unwrap_or((body, ""));
    Some(part_seconds(date_part, DATE_UNITS)? + part_seconds(time_part, TIME_UNITS)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hours_minutes_and_seconds_add_up() {
        assert_eq!(iso_seconds("PT2H44M"), Some(9_840.0));
        assert_eq!(iso_seconds("PT18H23M30S"), Some(66_210.0));
        assert_eq!(iso_seconds("PT45M30S"), Some(2_730.0));
    }

    #[test]
    fn fractional_seconds_and_days_are_read() {
        assert_eq!(iso_seconds("PT210.026S"), Some(210.026));
        assert_eq!(iso_seconds("P1DT1H"), Some(90_000.0));
    }

    #[test]
    fn anything_else_is_nothing() {
        assert_eq!(iso_seconds(""), None);
        assert_eq!(iso_seconds("2H"), None);
        assert_eq!(iso_seconds("PT5X"), None);
        assert_eq!(iso_seconds("PT12"), None);
    }
}
