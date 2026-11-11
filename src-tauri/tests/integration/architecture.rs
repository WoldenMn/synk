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

fn importers_of_commands(src: &Path) -> Vec<String> {
    let mut files = Vec::new();
    rust_files(src, &mut files);
    let commands_dir = src.join("commands");
    let lib = src.join("lib.rs");
    let mut offenders: Vec<String> = files
        .iter()
        .filter(|p| !p.starts_with(&commands_dir) && **p != lib)
        .filter(|p| {
            let text = std::fs::read_to_string(p).expect("lecture");
            text.contains("crate::commands")
                || text
                    .lines()
                    .any(|l| l.contains("crate::{") && l.contains("commands"))
        })
        .map(|p| {
            p.strip_prefix(src)
                .expect("prefixe")
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect();
    offenders.sort();
    offenders
}

fn http_client_builders(src: &Path) -> Vec<String> {
    let mut files = Vec::new();
    rust_files(src, &mut files);
    let common = src.join("utils").join("http.rs");
    let mut offenders: Vec<String> = files
        .iter()
        .filter(|p| **p != common)
        .filter(|p| {
            std::fs::read_to_string(p)
                .expect("lecture")
                .contains("Client::builder(")
        })
        .map(|p| {
            p.strip_prefix(src)
                .expect("prefixe")
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect();
    offenders.sort();
    offenders
}

const SHARED_MODULES: &[&str] = &["oauth", "store", "import"];

fn source_names(src: &Path) -> Vec<String> {
    let dir = src.join("sources");
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("dossier")
        .map(|e| e.expect("entree").path())
        .filter(|p| p.is_dir())
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .filter(|name| !SHARED_MODULES.contains(&name.as_str()))
        .collect();
    names.sort();
    names
}

fn importers_of_sources(src: &Path) -> Vec<String> {
    let mut files = Vec::new();
    rust_files(src, &mut files);
    let registry = src.join("sources").join("mod.rs");
    let lib = src.join("lib.rs");
    let mut offenders = Vec::new();
    for name in source_names(src) {
        let own_dir = src.join("sources").join(&name);
        for path in files
            .iter()
            .filter(|p| !p.starts_with(&own_dir) && **p != registry && **p != lib)
        {
            let text = std::fs::read_to_string(path).expect("lecture");
            let direct = text.contains(&format!("sources::{name}"));
            let grouped = text
                .lines()
                .any(|l| l.contains("sources::{") && l.contains(name.as_str()));
            if direct || grouped {
                let rel = path
                    .strip_prefix(src)
                    .expect("prefixe")
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                offenders.push(rel);
            }
        }
    }
    offenders.sort();
    offenders.dedup();
    offenders
}

#[test]
fn nothing_outside_a_source_imports_its_internals() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert!(
        !source_names(&src).is_empty(),
        "aucune source trouvee : le garde ne verrait rien"
    );
    let offenders = importers_of_sources(&src);
    assert!(
        offenders.is_empty(),
        "imports d'une source hors de son dossier et du registre : {offenders:?}"
    );
}

#[test]
fn the_source_guard_sees_a_planted_import() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let root = dir.path();
    for sub in [
        "commands",
        "services",
        "sources/withings",
        "sources/oura",
        "sources/oauth",
        "sources/store",
    ] {
        std::fs::create_dir_all(root.join(sub)).expect("dossier");
    }
    let files = [
        ("lib.rs", "use crate::sources::withings::auth;"),
        ("sources/mod.rs", "pub mod withings; use withings::X;"),
        (
            "sources/withings/a.rs",
            "use crate::sources::withings::client;",
        ),
        ("sources/oura/ok.rs", "use crate::sources::Source;"),
        (
            "sources/oauth/ok.rs",
            "use crate::sources::oauth::TokenSet;",
        ),
        (
            "sources/oura/shared.rs",
            "use crate::sources::oauth::TokenSet;",
        ),
        ("commands/shared.rs", "use crate::sources::{oauth, store};"),
        (
            "services/shared.rs",
            "use crate::sources::store::load_auth;",
        ),
        ("sources/oura/x.rs", "use crate::sources::withings::client;"),
        ("commands/ok.rs", "use crate::sources::{Domain, Source};"),
        ("commands/bad.rs", "use crate::sources::withings::auth;"),
        (
            "commands/bad2.rs",
            "use crate::sources::{withings, Domain};",
        ),
    ];
    for (name, text) in files {
        std::fs::write(root.join(name), text).expect("ecriture");
    }
    assert_eq!(
        importers_of_sources(root),
        vec!["commands/bad.rs", "commands/bad2.rs", "sources/oura/x.rs"]
    );
}

#[test]
fn nothing_outside_commands_imports_commands() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let offenders = importers_of_commands(&src);
    assert!(
        offenders.is_empty(),
        "imports remontants vers commands/ : {offenders:?}"
    );
}

#[test]
fn the_guard_sees_a_planted_import() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let root = dir.path();
    std::fs::create_dir_all(root.join("commands")).expect("dossier");
    std::fs::write(root.join("lib.rs"), "use crate::commands::x;").expect("ecriture");
    std::fs::write(
        root.join("commands").join("a.rs"),
        "use crate::commands::y;",
    )
    .expect("ecriture");
    std::fs::write(root.join("ok.rs"), "use crate::services::z;").expect("ecriture");
    std::fs::write(root.join("bad.rs"), "use crate::commands::metrics;").expect("ecriture");
    std::fs::write(root.join("bad2.rs"), "use crate::{commands, db};").expect("ecriture");
    assert_eq!(importers_of_commands(root), vec!["bad.rs", "bad2.rs"]);
}

#[test]
fn every_http_client_goes_through_the_common_constructor() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let offenders = http_client_builders(&src);
    assert!(
        offenders.is_empty(),
        "client HTTP construit hors de utils/http.rs : {offenders:?}"
    );
}

#[test]
fn the_http_guard_sees_a_planted_builder() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let root = dir.path();
    std::fs::create_dir_all(root.join("utils")).expect("dossier");
    std::fs::write(
        root.join("utils").join("http.rs"),
        "reqwest::Client::builder()",
    )
    .expect("ecriture");
    std::fs::write(root.join("ok.rs"), "crate::utils::http::client(base, t)").expect("ecriture");
    std::fs::write(root.join("bad.rs"), "let c = Client::builder().build();").expect("ecriture");
    assert_eq!(http_client_builders(root), vec!["bad.rs"]);
}

#[test]
fn table_names_interpolated_into_sql_must_be_static_literals() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (file, signature) in [
        (
            "commands/dashboard/metrics.rs",
            "async fn count_table(pool: &SqlitePool, table: &'static str)",
        ),
        ("commands/export/json/engine.rs", "table: &'static str,"),
    ] {
        let text = std::fs::read_to_string(src.join(file)).expect("lecture");
        assert!(
            text.contains(signature),
            "{file} : le nom de table interpole doit rester un &'static str"
        );
    }
}

#[test]
fn the_metric_services_do_not_depend_on_the_sources_that_feed_them() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src.join("services").join("metrics"), &mut files);
    assert!(!files.is_empty());
    let offenders: Vec<String> = files
        .iter()
        .filter(|p| {
            std::fs::read_to_string(p)
                .expect("lecture")
                .contains("crate::sources")
        })
        .map(|p| p.display().to_string())
        .collect();
    assert!(offenders.is_empty(), "{offenders:?}");
}

#[test]
fn the_generic_oauth_layer_names_no_brand_in_its_routing() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let flow = std::fs::read_to_string(src.join("sources/oauth/flow.rs")).expect("lecture");
    let production = flow.split("#[cfg(test)]").next().expect("code");
    assert!(
        !production.to_lowercase().contains("withings"),
        "la couche OAuth generique ne nomme aucune marque"
    );
}
