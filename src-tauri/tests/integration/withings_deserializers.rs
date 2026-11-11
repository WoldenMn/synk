use serde::Deserialize;
use synk_lib::sources::withings::types::{
    deserialize_heart_rate, deserialize_lenient_f64, deserialize_lenient_i64, deserialize_more,
};

#[derive(Deserialize)]
struct TestMore {
    #[serde(default, deserialize_with = "deserialize_more")]
    more: Option<i64>,
}

#[derive(Deserialize)]
struct TestLenientI64 {
    #[serde(default, deserialize_with = "deserialize_lenient_i64")]
    value: Option<i64>,
}

#[derive(Deserialize)]
struct TestLenientF64 {
    #[serde(default, deserialize_with = "deserialize_lenient_f64")]
    value: Option<f64>,
}

#[test]
fn more_accepts_int() {
    let json = r#"{"more": 1}"#;
    let t: TestMore = serde_json::from_str(json).unwrap();
    assert_eq!(t.more, Some(1));
}

#[test]
fn more_accepts_bool_true() {
    let json = r#"{"more": true}"#;
    let t: TestMore = serde_json::from_str(json).unwrap();
    assert_eq!(t.more, Some(1));
}

#[test]
fn more_accepts_bool_false() {
    let json = r#"{"more": false}"#;
    let t: TestMore = serde_json::from_str(json).unwrap();
    assert_eq!(t.more, Some(0));
}

#[test]
fn more_accepts_missing() {
    let json = r#"{}"#;
    let t: TestMore = serde_json::from_str(json).unwrap();
    assert_eq!(t.more, None);
}

#[test]
fn lenient_i64_accepts_int() {
    let json = r#"{"value": 42}"#;
    let t: TestLenientI64 = serde_json::from_str(json).unwrap();
    assert_eq!(t.value, Some(42));
}

#[test]
fn lenient_i64_ignores_string() {
    let json = r#"{"value": "not a number"}"#;
    let t: TestLenientI64 = serde_json::from_str(json).unwrap();
    assert_eq!(t.value, None);
}

#[test]
fn lenient_i64_ignores_object() {
    let json = r#"{"value": {"x": 1}}"#;
    let t: TestLenientI64 = serde_json::from_str(json).unwrap();
    assert_eq!(t.value, None);
}

#[test]
fn lenient_i64_accepts_null() {
    let json = r#"{"value": null}"#;
    let t: TestLenientI64 = serde_json::from_str(json).unwrap();
    assert_eq!(t.value, None);
}

#[test]
fn lenient_f64_accepts_float() {
    let json = r#"{"value": 2.5}"#;
    let t: TestLenientF64 = serde_json::from_str(json).unwrap();
    assert_eq!(t.value, Some(2.5));
}

#[test]
fn lenient_f64_accepts_int_as_float() {
    let json = r#"{"value": 42}"#;
    let t: TestLenientF64 = serde_json::from_str(json).unwrap();
    assert_eq!(t.value, Some(42.0));
}

// ── Piège 1 : night_events — string JSON → lenient_i64 retourne None ─────────
// (le piège documenté : Withings renvoie night_events comme une string JSON au lieu d'i64)

#[test]
fn night_events_as_string_is_ignored() {
    use synk_lib::sources::withings::types::SleepData;
    let json = r#"{"night_events": "[{\"ts\":1000,\"type\":2}]"}"#;
    let d: SleepData = serde_json::from_str(json).unwrap();
    assert_eq!(
        d.night_events, None,
        "string JSON dans night_events → lenient_i64 retourne None"
    );
}

#[test]
fn night_events_as_int_is_accepted() {
    use synk_lib::sources::withings::types::SleepData;
    let json = r#"{"night_events": 3}"#;
    let d: SleepData = serde_json::from_str(json).unwrap();
    assert_eq!(d.night_events, Some(3));
}

// ── Piège 2 : heart_rate — int brut OU struct ─────────────────────────────────

#[derive(Deserialize)]
struct TestHeartRate {
    #[serde(default, deserialize_with = "deserialize_heart_rate")]
    heart_rate: Option<synk_lib::sources::withings::types::HeartHr>,
}

#[test]
fn heart_rate_accepts_int() {
    let json = r#"{"heart_rate": 72}"#;
    let t: TestHeartRate = serde_json::from_str(json).unwrap();
    let hr = t.heart_rate.expect("devrait être Some");
    assert_eq!(hr.value, Some(72));
    assert_eq!(hr.qrs_duration, None);
}

#[test]
fn heart_rate_accepts_struct() {
    let json = r#"{"heart_rate": {"value": 68, "qrs_duration": 95}}"#;
    let t: TestHeartRate = serde_json::from_str(json).unwrap();
    let hr = t.heart_rate.expect("devrait être Some");
    assert_eq!(hr.value, Some(68));
    assert_eq!(hr.qrs_duration, Some(95));
}

#[test]
fn heart_rate_accepts_null() {
    let json = r#"{"heart_rate": null}"#;
    let t: TestHeartRate = serde_json::from_str(json).unwrap();
    assert!(t.heart_rate.is_none());
}

// ── Piège 4 : userid — int OU string ─────────────────────────────────────────

#[test]
fn userid_accepts_int() {
    use synk_lib::sources::withings::types::TokenBody;
    let json = r#"{
        "access_token": "acc",
        "refresh_token": "ref",
        "expires_in": 3600,
        "token_type": "Bearer",
        "userid": 12345
    }"#;
    let t: TokenBody = serde_json::from_str(json).unwrap();
    assert_eq!(t.userid, serde_json::json!(12345));
}

#[test]
fn userid_accepts_string() {
    use synk_lib::sources::withings::types::TokenBody;
    let json = r#"{
        "access_token": "acc",
        "refresh_token": "ref",
        "expires_in": 3600,
        "token_type": "Bearer",
        "userid": "12345"
    }"#;
    let t: TokenBody = serde_json::from_str(json).unwrap();
    assert_eq!(t.userid, serde_json::json!("12345"));
}

// ── Piège 5 : snoringepisodecount (orthographe exacte, PAS snoring_episode_count) ──

#[test]
fn snoringepisodecount_exact_spelling_accepted() {
    use synk_lib::sources::withings::types::SleepData;
    let json = r#"{"snoringepisodecount": 7}"#;
    let d: SleepData = serde_json::from_str(json).unwrap();
    assert_eq!(
        d.snoringepisodecount,
        Some(7),
        "orthographe exacte sans underscore"
    );
}

#[test]
fn snoringepisodecount_alias_spelling_accepted() {
    use synk_lib::sources::withings::types::SleepData;
    // L'alias snoring_episode_count est aussi valide (serde alias dans SleepData).
    let json = r#"{"snoring_episode_count": 4}"#;
    let d: SleepData = serde_json::from_str(json).unwrap();
    assert_eq!(
        d.snoringepisodecount,
        Some(4),
        "alias snoring_episode_count accepté"
    );
}

#[test]
fn snoringepisodecount_as_string_is_ignored() {
    use synk_lib::sources::withings::types::SleepData;
    // lenient_i64 : si Withings envoie une string → None sans erreur de désérialisation.
    let json = r#"{"snoringepisodecount": "oops"}"#;
    let d: SleepData = serde_json::from_str(json).unwrap();
    assert_eq!(
        d.snoringepisodecount, None,
        "string → lenient_i64 retourne None"
    );
}
