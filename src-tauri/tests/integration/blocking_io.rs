use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("dossier") {
        let path = entry.expect("entree").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn blank(bytes: &mut [u8], from: usize, to: usize) {
    for b in &mut bytes[from..to] {
        if *b != b'\n' {
            *b = b' ';
        }
    }
}

fn strip_literals(text: &str) -> Vec<u8> {
    let src = text.as_bytes();
    let mut out = src.to_vec();
    let mut i = 0;
    while i < src.len() {
        match src[i] {
            b'/' if src.get(i + 1) == Some(&b'/') => {
                let end = src[i..]
                    .iter()
                    .position(|b| *b == b'\n')
                    .map_or(src.len(), |p| i + p);
                blank(&mut out, i, end);
                i = end;
            }
            b'/' if src.get(i + 1) == Some(&b'*') => {
                let end = text[i + 2..]
                    .find("*/")
                    .map_or(src.len(), |p| i + 2 + p + 2);
                blank(&mut out, i, end);
                i = end;
            }
            b'"' => {
                let mut j = i + 1;
                while j < src.len() && src[j] != b'"' {
                    j += if src[j] == b'\\' { 2 } else { 1 };
                }
                let end = (j + 1).min(src.len());
                blank(&mut out, i + 1, j.min(src.len()));
                i = end;
            }
            b'\'' => {
                let close = if src.get(i + 1) == Some(&b'\\') {
                    src[i + 2..]
                        .iter()
                        .position(|b| *b == b'\'')
                        .map(|p| i + 2 + p)
                } else if src.get(i + 2) == Some(&b'\'') {
                    Some(i + 2)
                } else {
                    None
                };
                match close {
                    Some(c) => {
                        blank(&mut out, i + 1, c);
                        i = c + 1;
                    }
                    None => i += 1,
                }
            }
            _ => i += 1,
        }
    }
    out
}

fn matching(code: &[u8], open: usize, left: u8, right: u8) -> usize {
    let mut depth = 0;
    for (k, b) in code.iter().enumerate().skip(open) {
        if *b == left {
            depth += 1;
        } else if *b == right {
            depth -= 1;
            if depth == 0 {
                return k;
            }
        }
    }
    code.len() - 1
}

fn find_all(code: &[u8], needle: &str) -> Vec<usize> {
    let hay = String::from_utf8_lossy(code).into_owned();
    hay.match_indices(needle).map(|(p, _)| p).collect()
}

fn blank_regions(code: &mut [u8], marker: &str, left: u8, right: u8) {
    for at in find_all(code, marker) {
        let open = at + marker.len() - 1;
        if code.get(open) == Some(&left) {
            let close = matching(code, open, left, right);
            blank(code, at, close + 1);
        }
    }
}

const BLOCKING: &[&str] = &[
    "std::fs::",
    "OpenOptions::new()",
    "File::open(",
    "File::create(",
    "from_path(",
    "inspect_file(",
];

fn blocking_calls_in_async(text: &str) -> Vec<String> {
    let mut code = strip_literals(text);
    for at in find_all(&code.clone(), "#[cfg(test)]") {
        let rest = String::from_utf8_lossy(&code[at..]).into_owned();
        if let Some(p) = rest.find('{') {
            let open = at + p;
            let close = matching(&code, open, b'{', b'}');
            blank(&mut code, at, close + 1);
        }
    }
    blank_regions(&mut code, "spawn_blocking(", b'(', b')');
    blank_regions(&mut code, "retry_on_sharing_violation(", b'(', b')');
    let mut found = Vec::new();
    let mut bodies: Vec<(usize, usize)> = Vec::new();
    for at in find_all(&code, "async fn ") {
        let rest = &code[at..];
        let stop = rest.iter().position(|b| *b == b'{' || *b == b';');
        if let Some(p) = stop {
            if rest[p] == b'{' {
                bodies.push((at + p, matching(&code, at + p, b'{', b'}')));
            }
        }
    }
    for marker in ["async move {", "async {"] {
        for at in find_all(&code, marker) {
            let open = at + marker.len() - 1;
            bodies.push((open, matching(&code, open, b'{', b'}')));
        }
    }
    for (open, close) in bodies {
        let body = String::from_utf8_lossy(&code[open..=close]).into_owned();
        for pattern in BLOCKING {
            if body.contains(pattern) {
                let line = text[..open].matches('\n').count() + 1;
                found.push(format!("ligne {line} : {pattern}"));
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

#[test]
fn no_async_function_does_blocking_file_io() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    assert!(files.len() > 100, "le garde ne voit pas les sources");
    let mut offenders = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).expect("lecture");
        for call in blocking_calls_in_async(&text) {
            let rel = path.strip_prefix(&src).expect("prefixe");
            offenders.push(format!("{} {call}", rel.display()));
        }
    }
    offenders.sort();
    assert!(
        offenders.is_empty(),
        "appels de fichier bloquants dans une fonction async (tokio::fs ou spawn_blocking) :\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_guard_sees_planted_blocking_calls_and_spares_the_sanctioned_ones() {
    let bad = r#"
        async fn a() { let x = "{ pas une accolade"; std::fs::copy(p, q).unwrap(); }
        async fn b() { tokio::spawn(async move { std::fs::write(p, q).unwrap(); }); }
        async fn c() { let f = std::fs::File::open(p); }
        async fn d() { let w = WriterBuilder::new().from_path(p); }
        async fn e() { let i = inspect_file(p); }
    "#;
    assert_eq!(blocking_calls_in_async(bad).len(), 6, "{bad}");

    let good = r#"
        async fn a() {
            let n = tauri::async_runtime::spawn_blocking(move || std::fs::read(p)).await;
            retry_on_sharing_violation(|| std::fs::rename(a, b)).await;
            tokio::fs::copy(p, q).await;
            let s = "std::fs::write dans une chaine";
            // std::fs::write dans un commentaire
        }
        fn sync_helper() { std::fs::write(p, q).unwrap(); }
        #[cfg(test)]
        mod tests {
            async fn t() { std::fs::write(p, q).unwrap(); }
        }
    "#;
    assert_eq!(blocking_calls_in_async(good), Vec::<String>::new());
}
