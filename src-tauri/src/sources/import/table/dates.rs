use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde::Serialize;

pub const ISO: &str = "iso";
pub const UNIX: &str = "unix";
pub const UNIX_MS: &str = "unix_ms";

#[derive(Debug, Clone, Copy, Serialize)]
pub struct DateFormat {
    pub id: &'static str,
    pub label: &'static str,
    pub example: &'static str,
}

pub static FORMATS: &[DateFormat] = &[
    DateFormat {
        id: ISO,
        label: "Année-mois-jour (ISO)",
        example: "2026-10-25 07:30",
    },
    DateFormat {
        id: "%d/%m/%Y %H:%M:%S",
        label: "Jour/mois/année avec secondes",
        example: "25/10/2026 07:30:00",
    },
    DateFormat {
        id: "%d/%m/%Y %H:%M",
        label: "Jour/mois/année et heure",
        example: "25/10/2026 07:30",
    },
    DateFormat {
        id: "%d/%m/%Y",
        label: "Jour/mois/année",
        example: "25/10/2026",
    },
    DateFormat {
        id: "%m/%d/%Y %H:%M:%S",
        label: "Mois/jour/année avec secondes",
        example: "10/25/2026 07:30:00",
    },
    DateFormat {
        id: "%m/%d/%Y %H:%M",
        label: "Mois/jour/année et heure",
        example: "10/25/2026 07:30",
    },
    DateFormat {
        id: "%m/%d/%Y",
        label: "Mois/jour/année",
        example: "10/25/2026",
    },
    DateFormat {
        id: "%d-%m-%Y %H:%M",
        label: "Jour-mois-année et heure",
        example: "25-10-2026 07:30",
    },
    DateFormat {
        id: "%m-%d-%Y %H:%M",
        label: "Mois-jour-année et heure",
        example: "10-25-2026 07:30",
    },
    DateFormat {
        id: "%d.%m.%Y %H:%M",
        label: "Jour.mois.année et heure",
        example: "25.10.2026 07:30",
    },
    DateFormat {
        id: "%d.%m.%Y",
        label: "Jour.mois.année",
        example: "25.10.2026",
    },
    DateFormat {
        id: UNIX,
        label: "Secondes depuis 1970",
        example: "1792913400",
    },
    DateFormat {
        id: UNIX_MS,
        label: "Millisecondes depuis 1970",
        example: "1792913400000",
    },
];

pub fn is_known(format: &str) -> bool {
    FORMATS.iter().any(|f| f.id == format)
}

const ISO_DATE_TIME: &[&str] = &[
    "%Y-%m-%dT%H:%M:%S%.f",
    "%Y-%m-%d %H:%M:%S%.f",
    "%Y-%m-%dT%H:%M",
    "%Y-%m-%d %H:%M",
];

fn local_to_utc(naive: NaiveDateTime) -> Option<DateTime<Utc>> {
    Local
        .from_local_datetime(&naive)
        .earliest()
        .map(|t| t.with_timezone(&Utc))
}

fn noon(date: NaiveDate) -> Option<DateTime<Utc>> {
    local_to_utc(date.and_time(NaiveTime::from_hms_opt(12, 0, 0)?))
}

fn from_pattern(text: &str, pattern: &str) -> Option<DateTime<Utc>> {
    if let Ok(naive) = NaiveDateTime::parse_from_str(text, pattern) {
        return local_to_utc(naive);
    }
    NaiveDate::parse_from_str(text, pattern).ok().and_then(noon)
}

pub fn parse(text: &str, format: &str) -> Option<DateTime<Utc>> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    match format {
        UNIX => text
            .parse::<i64>()
            .ok()
            .and_then(|s| DateTime::from_timestamp(s, 0)),
        UNIX_MS => text
            .parse::<i64>()
            .ok()
            .and_then(DateTime::from_timestamp_millis),
        ISO => {
            if let Ok(t) = DateTime::parse_from_rfc3339(text) {
                return Some(t.with_timezone(&Utc));
            }
            ISO_DATE_TIME
                .iter()
                .find_map(|p| NaiveDateTime::parse_from_str(text, p).ok())
                .and_then(local_to_utc)
                .or_else(|| {
                    NaiveDate::parse_from_str(text, "%Y-%m-%d")
                        .ok()
                        .and_then(noon)
                })
        }
        pattern => from_pattern(text, pattern),
    }
}

pub fn guess<'a>(samples: impl Iterator<Item = &'a str> + Clone) -> Option<&'static str> {
    let non_empty: Vec<&str> = samples.filter(|s| !s.trim().is_empty()).collect();
    if non_empty.is_empty() {
        return None;
    }
    FORMATS
        .iter()
        .find(|f| non_empty.iter().all(|s| parse(s, f.id).is_some()))
        .map(|f| f.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(t: Option<DateTime<Utc>>) -> String {
        t.expect("date").format("%Y-%m-%dT%H:%MZ").to_string()
    }

    #[test]
    fn an_offset_is_kept_and_a_unix_time_is_exact() {
        assert_eq!(
            text(parse("2026-10-25T09:30:00+02:00", ISO)),
            "2026-10-25T07:30Z"
        );
        assert_eq!(
            text(parse("2026-10-25T07:30:00Z", ISO)),
            "2026-10-25T07:30Z"
        );
        assert_eq!(text(parse("1792913400", UNIX)), "2026-10-25T07:30Z");
        assert_eq!(text(parse("1792913400000", UNIX_MS)), "2026-10-25T07:30Z");
    }

    #[test]
    fn day_first_and_month_first_are_not_mixed_up() {
        let day_first = parse("05/10/2026 08:00", "%d/%m/%Y %H:%M").expect("date");
        let month_first = parse("05/10/2026 08:00", "%m/%d/%Y %H:%M").expect("date");
        assert_ne!(day_first, month_first);
        assert!(parse("25/10/2026 08:00", "%m/%d/%Y %H:%M").is_none());
    }

    #[test]
    fn a_date_without_a_time_lands_at_noon_local() {
        let t = parse("25/10/2026", "%d/%m/%Y").expect("date");
        let local = t.with_timezone(&Local);
        assert_eq!(
            local.format("%Y-%m-%d %H:%M").to_string(),
            "2026-10-25 12:00"
        );
    }

    #[test]
    fn the_format_is_guessed_from_samples_that_all_parse() {
        let rows = ["25/10/2026 07:30", "26/10/2026 08:00"];
        assert_eq!(guess(rows.iter().copied()), Some("%d/%m/%Y %H:%M"));
        let iso = ["2026-10-25 07:30:00", "2026-10-26T08:00:00"];
        assert_eq!(guess(iso.iter().copied()), Some(ISO));
        let ambiguous = ["05/10/2026 07:30", "06/10/2026 08:00"];
        assert_eq!(guess(ambiguous.iter().copied()), Some("%d/%m/%Y %H:%M"));
        let nothing = ["hier", "demain"];
        assert_eq!(guess(nothing.iter().copied()), None);
    }

    #[test]
    fn garbage_and_empty_text_give_nothing() {
        assert!(parse("", ISO).is_none());
        assert!(parse("pas une date", ISO).is_none());
        assert!(parse("12", UNIX_MS).is_some());
        assert!(parse("x", "%d/%m/%Y").is_none());
        assert!(is_known("unix") && !is_known("%Q"));
    }
}
