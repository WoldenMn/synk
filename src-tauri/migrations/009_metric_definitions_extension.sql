-- Migration 009 : etendre metric_definitions.category CHECK constraint et
-- ajouter 5 entries pour completion stack metriques V2 :
--   vital_bp        : systolic, diastolic, heart_rate
--   vital_sleep     : sleep_score
--   vital_activity  : steps
--
-- SQLite ne supporte pas ALTER TABLE ADD CHECK -> pattern rebuild officiel
-- (https://www.sqlite.org/lang_altertable.html#otheralter) :
-- (1) PRAGMA foreign_keys=OFF (sinon DROP TABLE echoue car metrics.definition_id la reference)
-- (2) CREATE TABLE new (nom temporaire)
-- (3) INSERT INTO new SELECT FROM old (preserve les id pour les FK)
-- (4) DROP TABLE old
-- (5) ALTER TABLE new RENAME TO old (le nom canonique)
-- (6) Recreer l'index
-- (7) INSERT les nouveaux slugs
-- (8) PRAGMA foreign_keys=ON
--
-- Cet ordre evite le bug "no such table metric_definitions_old" : on cree
-- la nouvelle table avec un nom temporaire, on copie, on supprime l'ancienne
-- (FK desactivees), puis on renomme la nouvelle au nom canonique.

PRAGMA foreign_keys=OFF;

-- Step 1 : Creer la nouvelle table sous un nom temporaire avec CHECK etendu
CREATE TABLE metric_definitions_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    unit TEXT NOT NULL,
    category TEXT NOT NULL CHECK(category IN (
        'body_composition',
        'circumference',
        'body_scan',
        'derived',
        'custom',
        'vital_bp',
        'vital_sleep',
        'vital_activity'
    )),
    withings_type INTEGER,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 100,
    decimals INTEGER NOT NULL DEFAULT 2,
    min_value REAL,
    max_value REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Step 2 : Copier les donnees (preserve les id pour les FK)
INSERT INTO metric_definitions_new
    (id, slug, label, unit, category, withings_type, is_builtin, sort_order, decimals, min_value, max_value, created_at)
SELECT
    id, slug, label, unit, category, withings_type, is_builtin, sort_order, decimals, min_value, max_value, created_at
FROM metric_definitions;

-- Step 3 : Supprimer l'ancienne table (FK OFF, donc pas de check sur metrics.definition_id)
DROP TABLE metric_definitions;

-- Step 4 : Renommer la nouvelle au nom canonique
ALTER TABLE metric_definitions_new RENAME TO metric_definitions;

-- Step 5 : Recreer l'index sur category
CREATE INDEX IF NOT EXISTS idx_metric_definitions_category ON metric_definitions(category);

-- Step 6 : INSERT les 5 nouveaux slugs
INSERT OR IGNORE INTO metric_definitions
    (slug, label, unit, category, withings_type, is_builtin, sort_order, decimals, min_value, max_value)
VALUES
    ('systolic',    'Tension systolique',  'mmHg', 'vital_bp',       10,   1, 50, 0, 50,  300),
    ('diastolic',   'Tension diastolique', 'mmHg', 'vital_bp',       9,    1, 51, 0, 30,  200),
    ('heart_rate',  'Frequence cardiaque', 'bpm',  'vital_bp',       11,   1, 52, 0, 30,  250),
    ('sleep_score', 'Score de sommeil',    'pts',  'vital_sleep',    NULL, 1, 60, 0, 0,   100),
    ('steps',       'Pas',                 'pas',  'vital_activity', NULL, 1, 70, 0, 0,   100000);

-- Step 7 : Reactiver les FK
PRAGMA foreign_keys=ON;
