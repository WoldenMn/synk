CREATE TABLE IF NOT EXISTS withings_credentials (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    client_id TEXT NOT NULL DEFAULT '',
    client_secret TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO withings_credentials (id, client_id, client_secret)
VALUES (1, '', '');
