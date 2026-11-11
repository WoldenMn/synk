use serde_json::Value;
use synk_lib::boot::PROD_IDENTIFIER;

fn identifier_of(file: &str) -> String {
    let path = format!("{}/{file}", env!("CARGO_MANIFEST_DIR"));
    let json: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("config"))
        .expect("config lisible");
    json["identifier"].as_str().expect("identifier").to_string()
}

#[test]
fn the_production_identifier_is_the_one_tauri_builds_with() {
    assert_eq!(identifier_of("tauri.conf.json"), PROD_IDENTIFIER);
}

#[test]
fn the_dev_config_is_the_production_identifier_with_a_dev_suffix() {
    let dev = identifier_of("tauri.dev.conf.json");
    assert_ne!(dev, PROD_IDENTIFIER);
    assert_eq!(dev, format!("{PROD_IDENTIFIER}.dev"));
}
