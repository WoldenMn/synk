const SECRET_KEYS: &[&str] = &[
    "access_token",
    "refresh_token",
    "client_secret",
    "code",
    "code_verifier",
    "id_token",
    "authorization",
];
pub const MASK: &str = "[masque]";

pub fn redact(text: &str) -> String {
    let src = text.as_bytes();
    let low = text.to_ascii_lowercase().into_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        match secret_at(&low, i) {
            Some((start, end)) => {
                out.extend_from_slice(&src[i..start]);
                out.extend_from_slice(MASK.as_bytes());
                i = end;
            }
            None => {
                out.push(src[i]);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| MASK.to_string())
}

fn secret_at(low: &[u8], i: usize) -> Option<(usize, usize)> {
    for key in SECRET_KEYS {
        let k = key.as_bytes();
        if low[i] == b'"' && low[i + 1..].starts_with(k) && low.get(i + 1 + k.len()) == Some(&b'"')
        {
            let colon = skip_spaces(low, i + 2 + k.len());
            if low.get(colon) != Some(&b':') {
                continue;
            }
            let quote = skip_spaces(low, colon + 1);
            if low.get(quote) != Some(&b'"') {
                continue;
            }
            return Some((quote + 1, string_end(low, quote + 1)));
        }
        let at_boundary =
            i == 0 || matches!(low[i - 1], b'?' | b'&' | b' ' | b'\t' | b'\n' | b'"' | b'(');
        if at_boundary && low[i..].starts_with(k) && low.get(i + k.len()) == Some(&b'=') {
            let start = i + k.len() + 1;
            return Some((
                start,
                until(low, start, |b| {
                    matches!(b, b'&' | b' ' | b'\t' | b'\n' | b'"' | b',' | b')')
                }),
            ));
        }
    }
    if low[i..].starts_with(b"bearer ") {
        let start = i + 7;
        return Some((
            start,
            until(low, start, |b| {
                matches!(b, b' ' | b'\t' | b'\n' | b'"' | b',')
            }),
        ));
    }
    None
}

fn skip_spaces(low: &[u8], mut j: usize) -> usize {
    while low.get(j).is_some_and(|b| b.is_ascii_whitespace()) {
        j += 1;
    }
    j
}

fn string_end(low: &[u8], start: usize) -> usize {
    let mut j = start;
    while j < low.len() {
        match low[j] {
            b'\\' => j += 2,
            b'"' => return j,
            _ => j += 1,
        }
    }
    low.len()
}

fn until(low: &[u8], start: usize, stop: impl Fn(u8) -> bool) -> usize {
    low[start..]
        .iter()
        .position(|&b| stop(b))
        .map_or(low.len(), |p| start + p)
}

#[cfg(test)]
mod tests {
    use super::{redact, MASK};

    #[test]
    fn json_tokens_are_masked_and_the_rest_kept() {
        let out = redact(
            r#"{"status":0,"body":{"access_token":"AT-SECRET-1","refresh_token":"RT-SECRET-2","expires_in":10800}}"#,
        );
        assert!(!out.contains("SECRET"), "{out}");
        assert!(out.contains(r#""expires_in":10800"#), "{out}");
        assert_eq!(out.matches(MASK).count(), 2, "{out}");
    }

    #[test]
    fn spaces_around_the_colon_do_not_hide_a_token() {
        assert!(!redact(r#"{ "access_token" : "AT-SECRET" }"#).contains("AT-SECRET"));
    }

    #[test]
    fn form_and_query_values_are_masked() {
        let out = redact("grant_type=refresh_token&refresh_token=RT-SECRET&client_secret=CS-SECRET&code=CODE-SECRET");
        assert!(!out.contains("SECRET"), "{out}");
        assert!(out.starts_with("grant_type=refresh_token&"), "{out}");
    }

    #[test]
    fn a_callback_url_inside_an_error_is_masked() {
        let out = redact("error sending request for url (http://127.0.0.1:17483/callback?code=CODE-SECRET&state=abc)");
        assert!(!out.contains("CODE-SECRET"), "{out}");
        assert!(out.contains("state=abc"), "{out}");
    }

    #[test]
    fn a_bearer_header_is_masked() {
        assert!(!redact("Authorization: Bearer AT-SECRET").contains("AT-SECRET"));
    }

    #[test]
    fn a_value_cut_by_truncation_is_masked_to_the_end() {
        assert!(!redact(r#"{"refresh_token":"RT-SEC"#).contains("RT-SEC"));
    }

    #[test]
    fn look_alikes_are_left_alone() {
        assert_eq!(
            redact("Withings code 401: refresh refused"),
            "Withings code 401: refresh refused"
        );
        assert_eq!(redact(r#"{"error_code":"E1"}"#), r#"{"error_code":"E1"}"#);
    }

    #[test]
    fn accents_survive_and_accented_secrets_are_masked() {
        assert_eq!(redact(r#"{"note":"évité"}"#), r#"{"note":"évité"}"#);
        let out = redact(r#"{"code":"é-secret"}"#);
        assert!(!out.contains("secret"), "{out}");
    }

    #[test]
    fn a_pkce_verifier_and_an_id_token_are_masked_in_json_and_form() {
        let out = redact(
            r#"{"code_verifier":"VER-SECRET","id_token":"ID-SECRET"} code_verifier=VER-SECRET2&id_token=ID-SECRET2"#,
        );
        assert!(!out.contains("SECRET"), "{out}");
    }
}
