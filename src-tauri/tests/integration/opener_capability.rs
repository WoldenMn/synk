use serde_json::Value;
use synk_lib::sources::{all_sources, SourceKind};

fn allowed_urls() -> Vec<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/capabilities/default.json");
    let json: Value = serde_json::from_str(&std::fs::read_to_string(path).expect("capacite"))
        .expect("capacite lisible");
    json["permissions"]
        .as_array()
        .expect("permissions")
        .iter()
        .filter(|p| p["identifier"] == "opener:allow-open-url")
        .flat_map(|p| p["allow"].as_array().cloned().unwrap_or_default())
        .filter_map(|a| a["url"].as_str().map(str::to_string))
        .collect()
}

fn origin(url: &str) -> String {
    let rest = url.strip_prefix("https://").expect("https");
    format!("https://{}/*", rest.split('/').next().expect("hote"))
}

#[test]
fn every_link_a_source_opens_is_allowed_by_its_exact_host() {
    let allowed = allowed_urls();
    for source in all_sources() {
        let mut links = Vec::new();
        if let SourceKind::OAuth(spec) = source.kind() {
            links.push(spec.authorize_url);
        }
        links.extend(source.developer_url());
        for link in links {
            let needed = origin(link);
            assert!(
                allowed.contains(&needed),
                "{} ouvre {link} sans entree {needed} dans opener:allow-open-url",
                source.label()
            );
        }
    }
}

#[test]
fn the_allowed_links_are_exact_hosts_never_wildcards() {
    for url in allowed_urls() {
        assert!(url.starts_with("https://") && url.ends_with("/*"), "{url}");
        let host = url.trim_start_matches("https://").trim_end_matches("/*");
        assert!(!host.contains('*') && !host.contains('/'), "{url}");
    }
}
