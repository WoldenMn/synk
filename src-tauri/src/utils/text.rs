pub fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::truncate_chars;

    #[test]
    fn counts_characters_not_bytes() {
        assert_eq!(truncate_chars("ééé", 2), "éé");
        assert_eq!(truncate_chars("court", 200), "court");
        assert_eq!(truncate_chars("", 3), "");
    }
}
