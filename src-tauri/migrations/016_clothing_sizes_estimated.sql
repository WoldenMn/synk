-- Migration 016 : autoriser la source 'estimated' sur clothing_sizes.
-- SQLite ne sait pas modifier un CHECK, on reconstruit la table
-- selon le pattern officiel deja applique par la migration 009.
PRAGMA foreign_keys=OFF;
CREATE TABLE clothing_sizes_new (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    category   TEXT NOT NULL UNIQUE,
    value      TEXT NOT NULL,
    source     TEXT NOT NULL CHECK (source IN ('override', 'manual', 'estimated')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO clothing_sizes_new (id, category, value, source, updated_at)
    SELECT id, category, value, source, updated_at FROM clothing_sizes;
DROP TABLE clothing_sizes;
ALTER TABLE clothing_sizes_new RENAME TO clothing_sizes;
PRAGMA foreign_keys=ON;
