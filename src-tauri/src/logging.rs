use log::LevelFilter;

pub const ENV_VAR: &str = "SYNK_LOG";
pub const FILES_KEPT: usize = 5;
pub const FILE_MAX_BYTES: u128 = 1_000_000;

pub fn level_from(value: Option<&str>) -> (LevelFilter, Option<String>) {
    match value {
        None => (LevelFilter::Info, None),
        Some(raw) => match raw.trim().parse::<LevelFilter>() {
            Ok(level) => (level, None),
            Err(_) => (
                LevelFilter::Info,
                Some(format!(
                    "{ENV_VAR}={raw} illisible (error, warn, info, debug, trace) : niveau info"
                )),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::level_from;
    use log::LevelFilter;

    #[test]
    fn info_by_default() {
        assert_eq!(level_from(None), (LevelFilter::Info, None));
    }

    #[test]
    fn reads_a_valid_level() {
        assert_eq!(level_from(Some("debug")).0, LevelFilter::Debug);
        assert_eq!(level_from(Some("TRACE")).0, LevelFilter::Trace);
        assert_eq!(level_from(Some(" warn ")), (LevelFilter::Warn, None));
    }

    #[test]
    fn an_unreadable_value_keeps_info_and_says_so() {
        let (level, warning) = level_from(Some("bavard"));
        assert_eq!(level, LevelFilter::Info);
        assert!(warning.expect("avertissement").contains("bavard"));
    }

    #[test]
    fn an_empty_value_is_unreadable_not_silent() {
        let (level, warning) = level_from(Some(""));
        assert_eq!(level, LevelFilter::Info);
        assert!(warning.is_some());
    }
}
