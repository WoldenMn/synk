-- Migration 019 : tables des sources (R33, premiere moitie).
--
-- Inventaire relu le 2026-10-25 (spec de R28, tables vivantes apres la 018) :
--   body_measures_legacy, vitals_legacy, blood_pressure, sleep_records, activity,
--   workouts, heart_records, metrics portent une colonne source.
--   body_measures, vitals, manual_measurements, measurement_types sont des vues
--   sur les tables _legacy (manual_measurements_legacy n'a pas de colonne source).
--   Source de verite par metrique : sa table historique, sauf heart_rate
--   (blood_pressure plus les lignes de metrics sans jumelle) et les six
--   derivees (metrics).
--
-- Ce qui change ici : une table sources, les tables de jetons, d'identifiants,
-- de curseurs et de priorite par source. La 020 pose la cle etrangere et
-- external_id sur les tables de mesures. withings_auth,
-- withings_credentials et sync_state deviennent des vues de compatibilite sur
-- ces tables, leurs lignes y sont recopiees. Rien n'est supprime : chaque
-- ligne est recopiee avec son identifiant.
--
-- Les anciennes tables _v2, _repair et _tmp (inventaire de R28) ne sont pas
-- touchees. Cette migration suppose les noms _legacy : le demarrage les pose
-- avant elle.

PRAGMA foreign_keys=OFF;

CREATE TABLE sources (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('oauth', 'file', 'internal')),
    sort_order INTEGER NOT NULL DEFAULT 100,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO sources (id, label, kind, sort_order) VALUES
    ('withings', 'Withings', 'oauth', 10),
    ('manual', 'Saisie manuelle', 'internal', 20),
    ('derived', 'Calculé', 'internal', 30),
    ('import', 'Import', 'file', 40),
    ('oura', 'Oura', 'oauth', 50),
    ('polar', 'Polar', 'oauth', 60),
    ('whoop', 'Whoop', 'oauth', 70),
    ('google_health', 'Google Health', 'oauth', 80),
    ('apple_health', 'Apple Santé', 'file', 90),
    ('fit', 'Fichiers FIT', 'file', 100),
    ('csv', 'Fichiers CSV', 'file', 110);

INSERT OR IGNORE INTO sources (id, label, kind, sort_order)
    SELECT DISTINCT source, source, 'internal', 900 FROM (
        SELECT source FROM body_measures_legacy
        UNION SELECT source FROM vitals_legacy
        UNION SELECT source FROM blood_pressure
        UNION SELECT source FROM sleep_records
        UNION SELECT source FROM activity
        UNION SELECT source FROM workouts
        UNION SELECT source FROM heart_records
        UNION SELECT source FROM metrics
    ) WHERE source IS NOT NULL AND source != '';

CREATE TABLE source_credentials (
    source_id TEXT PRIMARY KEY REFERENCES sources(id),
    client_id TEXT NOT NULL DEFAULT '',
    client_secret TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO source_credentials (source_id, client_id, client_secret, updated_at)
    SELECT 'withings', client_id, client_secret, updated_at FROM withings_credentials WHERE id = 1;

DROP TABLE withings_credentials;

CREATE VIEW withings_credentials AS
    SELECT 1 AS id, client_id, client_secret, updated_at
    FROM source_credentials WHERE source_id = 'withings';

CREATE TABLE source_auth (
    source_id TEXT PRIMARY KEY REFERENCES sources(id),
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    token_type TEXT NOT NULL DEFAULT 'Bearer',
    user_ref TEXT DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO source_auth (source_id, access_token, refresh_token, expires_at, token_type, user_ref, updated_at, created_at)
    SELECT 'withings', access_token, refresh_token, expires_at, token_type, withings_user_id, updated_at, created_at
    FROM withings_auth WHERE id = 1;

DROP TABLE withings_auth;

CREATE VIEW withings_auth AS
    SELECT 1 AS id, access_token, refresh_token, expires_at, token_type,
           user_ref AS withings_user_id, created_at, updated_at
    FROM source_auth WHERE source_id = 'withings';

CREATE TABLE source_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id TEXT NOT NULL REFERENCES sources(id),
    domain TEXT NOT NULL,
    last_sync_at INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (source_id, domain)
);

INSERT INTO source_state (id, source_id, domain, last_sync_at, updated_at)
    SELECT id, 'withings', data_type, last_sync_at, updated_at FROM sync_state;

DROP TABLE sync_state;

CREATE VIEW sync_state AS
    SELECT id, domain AS data_type, last_sync_at, updated_at
    FROM source_state WHERE source_id = 'withings';

CREATE TABLE metric_source_priority (
    definition_id INTEGER NOT NULL REFERENCES metric_definitions(id) ON DELETE CASCADE,
    source_id TEXT NOT NULL REFERENCES sources(id),
    rank INTEGER NOT NULL,
    PRIMARY KEY (definition_id, source_id),
    UNIQUE (definition_id, rank)
);

PRAGMA foreign_keys=ON;
