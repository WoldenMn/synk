use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use reqwest::Method;
use serde::Deserialize;
use synk_lib::error::AppError;
use synk_lib::sources::oauth::brand::{keep_going, plain_throttle, Brand};
use synk_lib::sources::oauth::rest::RestClient;
use synk_lib::sources::oauth::TokenSet;
use synk_lib::sources::{store, SourceId};

use crate::common::{memory_pool, test_vault};
use crate::support::fake_server::{FakeServer, Reply};

#[derive(Deserialize)]
struct Chunk {
    items: Vec<i32>,
    next: Option<String>,
}

static BRAND: Brand = Brand {
    label: "Essai",
    tag: "essai",
    forbidden: "Essai refuse l'acces",
    unauthorized: "Essai : jeton refuse",
    rate_label: "Essai : limite de debit atteinte",
    wait_headers: &["retry-after"],
    max_pages: 3,
    screen: keep_going,
    throttle: plain_throttle,
};

async fn client() -> RestClient {
    let pool = memory_pool().await;
    let vault = test_vault();
    let tokens = TokenSet {
        access_token: "access-essai".into(),
        refresh_token: "refresh-essai".into(),
        expires_in: 86400,
        token_type: "bearer".into(),
        user_ref: String::new(),
    };
    store::save_auth(&pool, vault.as_ref(), SourceId::OURA, &tokens)
        .await
        .unwrap();
    RestClient::connect(
        &BRAND,
        SourceId::OURA,
        "http://127.0.0.1",
        &pool,
        &vault,
        None,
    )
    .await
    .unwrap()
}

fn split(chunk: Chunk) -> (Vec<i32>, Option<String>) {
    (chunk.items, chunk.next)
}

#[tokio::test]
async fn pages_are_followed_until_the_token_stops_or_repeats() {
    let served = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&served);
    let fake = FakeServer::start_replying(move |call| {
        counter.fetch_add(1, Ordering::SeqCst);
        let body = match call.query_value("t").as_deref() {
            None => r#"{"items":[1],"next":"a"}"#,
            Some("a") => r#"{"items":[2],"next":"b"}"#,
            _ => r#"{"items":[3],"next":"b"}"#,
        };
        Reply::new(200, body)
    });
    let mut rest = client().await;
    let items = rest
        .pages(&format!("{}/x?q=1", fake.base_url), "t", "essai", split)
        .await
        .unwrap();
    assert_eq!(items, vec![1, 2, 3]);
    assert_eq!(served.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn an_endless_collection_stops_at_the_page_ceiling_with_a_named_error() {
    let fake = FakeServer::start_replying(move |call| {
        let n: usize = call
            .query_value("t")
            .and_then(|t| t.parse().ok())
            .unwrap_or(0);
        Reply::new(200, format!(r#"{{"items":[1],"next":"{}"}}"#, n + 1))
    });
    let mut rest = client().await;
    let err = rest
        .pages(&format!("{}/x?q=1", fake.base_url), "t", "essai", split)
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("plus de 3 pages pour essai"),
        "{err}"
    );
}

#[tokio::test]
async fn a_server_error_is_retried_then_a_refusal_is_named_without_the_body() {
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&hits);
    let fake = FakeServer::start_replying(move |call| {
        match (call.route(), counter.fetch_add(1, Ordering::SeqCst)) {
            ("/flaky", 0) => Reply::new(503, "FUITE secret"),
            ("/flaky", _) => Reply::new(200, r#"{"items":[],"next":null}"#),
            ("/forbidden", _) => Reply::new(403, "FUITE secret"),
            _ => Reply::new(401, "FUITE secret"),
        }
    });
    let mut rest = client().await;
    let ok = rest
        .send(Method::GET, &format!("{}/flaky", fake.base_url), None)
        .await
        .unwrap();
    assert_eq!(ok.status.as_u16(), 200);

    let forbidden = rest
        .send(Method::GET, &format!("{}/forbidden", fake.base_url), None)
        .await
        .unwrap_err();
    assert!(matches!(forbidden, AppError::Forbidden(_)), "{forbidden}");
    assert!(!forbidden.to_string().contains("FUITE"));

    let refused = rest
        .send(Method::GET, &format!("{}/other", fake.base_url), None)
        .await
        .unwrap_err();
    assert!(refused.to_string().contains("jeton refuse"), "{refused}");
    assert!(!refused.to_string().contains("FUITE"));
}

#[tokio::test]
async fn the_http_client_refuses_clear_text_for_an_https_base_and_never_follows_a_redirect() {
    use std::time::Duration;
    use synk_lib::utils::http::client;

    let fake = FakeServer::start_replying(|call| match call.route() {
        "/moved" => Reply::new(302, "").with_header("Location", "/elsewhere"),
        _ => Reply::new(200, "arrive"),
    });

    let strict = client("https://api.example.test", Duration::from_secs(5)).unwrap();
    let err = strict
        .get(format!("{}/moved", fake.base_url))
        .send()
        .await
        .unwrap_err();
    assert!(err.is_builder(), "{err}");
    assert!(fake.calls_on("/moved").is_empty());

    let lenient = client(&fake.base_url, Duration::from_secs(5)).unwrap();
    let reply = lenient
        .get(format!("{}/moved", fake.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(reply.status().as_u16(), 302);
    assert!(fake.calls_on("/elsewhere").is_empty());
}
