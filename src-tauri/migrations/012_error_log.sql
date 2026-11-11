CREATE TABLE IF NOT EXISTS error_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    occurred_at TEXT NOT NULL DEFAULT (datetime('now')),
    source      TEXT NOT NULL,
    code        TEXT NOT NULL,
    message     TEXT NOT NULL,
    context     TEXT
);
CREATE INDEX IF NOT EXISTS idx_error_log_occurred ON error_log(occurred_at);
