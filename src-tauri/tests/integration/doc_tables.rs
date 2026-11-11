#[tokio::test]
async fn documented_table_count_matches_the_schema() {
    let pool = crate::common::memory_pool().await;
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_one(&pool)
    .await
    .expect("compte");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let cited = cited_table_counts(&root);
    assert!(
        !cited.is_empty(),
        "aucune doc ne cite de nombre de tables et vues"
    );
    for (path, n) in cited {
        assert_eq!(
            n, count,
            "{path} cite {n} tables/vues, le schema en compte {count}"
        );
    }
}

#[test]
fn the_reader_finds_a_count_in_prose() {
    let dir = std::env::temp_dir().join(format!("synk-doc-tables-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("docs")).unwrap();
    std::fs::write(dir.join("README.md"), "voir 12 tables/vues ici").unwrap();
    std::fs::write(dir.join("CLAUDE.md"), "rien").unwrap();
    let found = cited_table_counts(&dir);
    std::fs::remove_dir_all(&dir).ok();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].1, 12);
}

#[test]
fn the_reader_does_without_claude_md() {
    let dir = std::env::temp_dir().join(format!("synk-doc-tables-sans-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("docs")).unwrap();
    std::fs::write(dir.join("README.md"), "voir 12 tables/vues ici").unwrap();
    let found = cited_table_counts(&dir);
    std::fs::remove_dir_all(&dir).ok();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].1, 12);
}

fn cited_table_counts(root: &std::path::Path) -> Vec<(String, i64)> {
    let mut found = Vec::new();
    let mut files = vec![root.join("README.md")];
    let claude = root.join("CLAUDE.md");
    if claude.is_file() {
        files.push(claude);
    }
    for entry in std::fs::read_dir(root.join("docs")).expect("docs") {
        let p = entry.expect("entree").path();
        if p.extension().and_then(|e| e.to_str()) == Some("md")
            && !p.ends_with("historique-developpement.md")
        {
            files.push(p);
        }
    }
    for path in files {
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} illisible : {e}", path.display()));
        for (i, _) in text.match_indices(" tables/vues") {
            let head = &text[..i];
            let digits: String = head
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            if let Ok(n) = digits.parse::<i64>() {
                found.push((path.display().to_string(), n));
            }
        }
    }
    found
}
