use synk_lib::commands::journal::{upsert_journal_entry_inner, JournalInput};

fn entry(date: &str, content: &str) -> JournalInput {
    JournalInput {
        date: date.to_string(),
        content: content.to_string(),
        tags: None,
    }
}

#[tokio::test]
async fn upsert_returns_the_id_of_the_row_it_wrote() {
    let pool = crate::common::memory_pool().await;

    let first = upsert_journal_entry_inner(&pool, entry("2026-06-01", "a"))
        .await
        .unwrap();
    let other = upsert_journal_entry_inner(&pool, entry("2026-06-02", "b"))
        .await
        .unwrap();
    let again = upsert_journal_entry_inner(&pool, entry("2026-06-01", "c"))
        .await
        .unwrap();

    assert_ne!(first, other);
    assert_eq!(again, first);
}

#[tokio::test]
async fn upsert_replaces_the_content_of_the_same_day() {
    let pool = crate::common::memory_pool().await;

    upsert_journal_entry_inner(&pool, entry("2026-06-01", "a"))
        .await
        .unwrap();
    upsert_journal_entry_inner(&pool, entry("2026-06-01", "c"))
        .await
        .unwrap();

    let rows: Vec<String> = sqlx::query_scalar("SELECT content FROM journal_entries")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(rows, vec!["c".to_string()]);
}

#[tokio::test]
async fn upsert_refuses_an_invalid_date_with_a_validation_error() {
    let pool = crate::common::memory_pool().await;

    let err = upsert_journal_entry_inner(&pool, entry("pas-une-date", "a"))
        .await
        .unwrap_err();

    assert_eq!(err.kind(), "Validation");
}
