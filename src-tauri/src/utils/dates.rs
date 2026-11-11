use crate::error::AppError;

/// Retourne les 10 premiers caracteres d'une date ISO (YYYY-MM-DD).
/// Decoupe par caractere, jamais par octet : aucune panique sur une entree non ASCII.
pub fn date_prefix(s: &str) -> &str {
    match s.char_indices().nth(10) {
        Some((i, _)) => s.get(..i).unwrap_or(s),
        None => s,
    }
}

/// Borne haute exclusive d'une plage : le jour qui suit `to`, en YYYY-MM-DD.
///
/// Les colonnes stockent des horodatages (`2026-09-13T06:30:00Z`). Comparees en
/// texte a `<= '2026-09-13'`, elles excluent tout le dernier jour ; `< '2026-09-14'`
/// les inclut, et reste juste pour les colonnes qui ne portent qu'une date.
pub fn day_after(to: &str) -> Result<String, AppError> {
    chrono::NaiveDate::parse_from_str(date_prefix(to), "%Y-%m-%d")
        .ok()
        .and_then(|d| d.succ_opt())
        .map(|d| d.format("%Y-%m-%d").to_string())
        .ok_or_else(|| AppError::Validation(format!("date de fin invalide : {to}")))
}

pub struct DayRange {
    pub from: String,
    pub before: String,
}

/// Bornes validees d'une plage de lecture : le premier jour (`from`, YYYY-MM-DD)
/// et le jour qui suit le dernier (`before`, exclusif). Une borne illisible,
/// de debut comme de fin, est une `Validation`, jamais une plage vide ou entiere.
pub fn day_range(from: &str, to: &str) -> Result<DayRange, AppError> {
    let first = chrono::NaiveDate::parse_from_str(date_prefix(from), "%Y-%m-%d")
        .map(|d| d.format("%Y-%m-%d").to_string())
        .map_err(|_| AppError::Validation(format!("date de debut invalide : {from}")))?;
    Ok(DayRange {
        from: first,
        before: day_after(to)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_after_skips_to_the_next_calendar_day() {
        assert_eq!(
            day_after("2026-09-13T13:05:00+02:00").unwrap(),
            "2026-09-14"
        );
        assert_eq!(day_after("2026-02-28").unwrap(), "2026-03-01");
        assert_eq!(day_after("2026-12-31T23:59:59Z").unwrap(), "2027-01-01");
    }

    #[test]
    fn day_after_rejects_an_unparsable_date() {
        assert!(day_after("pas-une-date").is_err());
        assert!(day_after("").is_err());
    }

    #[test]
    fn a_range_follows_the_utc_day_not_the_local_one() {
        let from = date_prefix("2026-09-03T00:00:00+02:00");
        let reading_at_1am_in_paris = "2026-09-02T23:00:00Z";
        assert!(reading_at_1am_in_paris < from);
        let reading_at_3am_in_paris = "2026-09-03T01:00:00Z";
        assert!(reading_at_3am_in_paris >= from);
    }

    #[test]
    fn truncates_iso_datetime() {
        assert_eq!(date_prefix("2026-04-24T08:30:00Z"), "2026-04-24");
    }

    #[test]
    fn truncates_iso_with_offset() {
        assert_eq!(date_prefix("2026-04-24T08:30:00+02:00"), "2026-04-24");
    }

    #[test]
    fn passes_through_date_only() {
        assert_eq!(date_prefix("2026-04-24"), "2026-04-24");
    }

    #[test]
    fn handles_shorter_than_10() {
        assert_eq!(date_prefix("2026"), "2026");
    }

    #[test]
    fn multibyte_input_does_not_panic() {
        assert_eq!(date_prefix("2026-09-0é suite"), "2026-09-0é");
        assert_eq!(date_prefix("ééééééééééé"), "éééééééééé");
    }

    #[test]
    fn handles_empty_string() {
        assert_eq!(date_prefix(""), "");
    }

    fn rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("dossier src lisible") {
            let path = entry.expect("entree lisible").path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    #[test]
    fn no_inclusive_upper_bound_left_in_range_queries() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let allowed = ["alerts.rs", "journal.rs", "dates.rs"];
        let mut files = Vec::new();
        rust_files(&src, &mut files);
        assert!(
            files.len() > 50,
            "la sonde ne voit que {} fichiers",
            files.len()
        );

        let offenders: Vec<String> = files
            .iter()
            .filter(|p| !allowed.iter().any(|a| p.ends_with(a)))
            .filter_map(|p| {
                let text = std::fs::read_to_string(p).expect("fichier lisible");
                let hits = text.matches("<= ?").count() + text.matches("BETWEEN ?").count();
                (hits > 0).then(|| format!("{} ({hits})", p.display()))
            })
            .collect();

        assert!(
            offenders.is_empty(),
            "borne inclusive sur une date tronquee (<= ? ou BETWEEN), utiliser >= date_prefix et < day_after : {offenders:#?}"
        );
    }
}
