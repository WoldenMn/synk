CREATE TABLE IF NOT EXISTS clothing_sizes (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    category   TEXT NOT NULL UNIQUE,
    value      TEXT NOT NULL,
    source     TEXT NOT NULL CHECK (source IN ('override', 'manual')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
