-- Migration 010 : etendre sync_state.data_type CHECK pour inclure 'metrics'.
--
-- Le stack metriques V2 (sync_metrics) persiste son curseur via la ligne
-- data_type='metrics', mais le CHECK de la migration 002 ne l'autorisait pas
-- -> INSERT rejete -> curseur jamais persiste -> sync_metrics repart de epoch
-- a chaque cycle (full re-sync perpetuel, bug #5).
--
-- SQLite ne supporte pas ALTER TABLE ... ADD CHECK -> pattern rebuild officiel
-- (identique aux migrations 002 et 009).

PRAGMA foreign_keys=OFF;

CREATE TABLE sync_state_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    data_type TEXT NOT NULL UNIQUE CHECK (data_type IN ('measures', 'activity', 'sleep', 'heart', 'workouts', 'devices', 'metrics')),
    last_sync_at INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO sync_state_new (id, data_type, last_sync_at, updated_at)
    SELECT id, data_type, last_sync_at, updated_at FROM sync_state;

DROP TABLE sync_state;

ALTER TABLE sync_state_new RENAME TO sync_state;

INSERT OR IGNORE INTO sync_state (data_type, last_sync_at) VALUES ('metrics', 0);

PRAGMA foreign_keys=ON;
