-- =============================================
-- Synk V2 migration 005 : Metriques unifiees
-- =============================================
-- Cree les 4 tables : metric_definitions, metrics, metric_targets, derived_metrics
-- Les renames *_legacy et vues de retrocompat sont geres en Rust (ensure_critical_tables (fonction retiree depuis, le schema vient des seules migrations))

-- 1. Tables principales

CREATE TABLE IF NOT EXISTS metric_definitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    unit TEXT NOT NULL,
    category TEXT NOT NULL CHECK(category IN ('body_composition','circumference','body_scan','derived','custom')),
    withings_type INTEGER,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    decimals INTEGER NOT NULL DEFAULT 2,
    min_value REAL,
    max_value REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_metric_definitions_category ON metric_definitions(category);
CREATE INDEX IF NOT EXISTS idx_metric_definitions_withings_type ON metric_definitions(withings_type) WHERE withings_type IS NOT NULL;

CREATE TABLE IF NOT EXISTS metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    definition_id INTEGER NOT NULL REFERENCES metric_definitions(id) ON DELETE CASCADE,
    measured_at TEXT NOT NULL,
    value REAL NOT NULL,
    source TEXT NOT NULL CHECK(source IN ('withings','manual','derived','import')),
    external_id TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(definition_id, measured_at, source, external_id)
);
CREATE INDEX IF NOT EXISTS idx_metrics_definition_date ON metrics(definition_id, measured_at);
CREATE INDEX IF NOT EXISTS idx_metrics_source ON metrics(source);
CREATE INDEX IF NOT EXISTS idx_metrics_external ON metrics(external_id) WHERE external_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS metric_targets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    definition_id INTEGER NOT NULL UNIQUE REFERENCES metric_definitions(id) ON DELETE CASCADE,
    target_value REAL,
    direction TEXT CHECK(direction IN ('decrease','increase','maintain','range')),
    threshold_warn_low REAL,
    threshold_warn_high REAL,
    threshold_danger_low REAL,
    threshold_danger_high REAL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS derived_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    definition_id INTEGER NOT NULL REFERENCES metric_definitions(id) ON DELETE CASCADE,
    measured_at TEXT NOT NULL,
    value REAL NOT NULL,
    formula_slug TEXT NOT NULL,
    source_metric_ids TEXT NOT NULL, -- JSON array d'ids de metrics (ex: [12,34])
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(definition_id, measured_at, formula_slug)
);
CREATE INDEX IF NOT EXISTS idx_derived_metrics_definition_date ON derived_metrics(definition_id, measured_at);
CREATE INDEX IF NOT EXISTS idx_derived_metrics_formula ON derived_metrics(formula_slug);

-- 2. Seed metric_definitions (built-in)

-- 2a. body_composition (8 entries, withings types mapped)
INSERT OR IGNORE INTO metric_definitions (slug, label, unit, category, withings_type, is_builtin, sort_order, decimals, min_value, max_value) VALUES
    ('weight',          'Poids',              'kg',     'body_composition', 1,    1, 10, 2, 20,  300),
    ('fat_free_mass',   'Masse maigre',       'kg',     'body_composition', 5,    1, 11, 2, 10,  200),
    ('fat_ratio',       'Masse grasse (%)',   '%',      'body_composition', 6,    1, 12, 2, 1,   70),
    ('fat_mass',        'Masse grasse',       'kg',     'body_composition', 8,    1, 13, 2, 1,   150),
    ('muscle_mass',     'Masse musculaire',   'kg',     'body_composition', 76,   1, 14, 2, 5,   150),
    ('hydration',       'Hydratation',        'kg',     'body_composition', 77,   1, 15, 2, 5,   100),
    ('bone_mass',       'Masse osseuse',      'kg',     'body_composition', 88,   1, 16, 2, 0.5, 10),
    ('bmi',             'IMC',                'kg/m2',  'body_composition', NULL, 1, 17, 2, 10,  60);

-- 2b. circumference (11 entries, manual uniquement)
INSERT OR IGNORE INTO metric_definitions (slug, label, unit, category, withings_type, is_builtin, sort_order, decimals, min_value, max_value) VALUES
    ('waist',         'Tour de taille',     'cm', 'circumference', NULL, 1, 20, 1, 30,  200),
    ('chest',         'Tour de poitrine',   'cm', 'circumference', NULL, 1, 21, 1, 50,  200),
    ('hips',          'Tour de hanches',    'cm', 'circumference', NULL, 1, 22, 1, 50,  200),
    ('shoulders',     'Tour d''epaules',    'cm', 'circumference', NULL, 1, 23, 1, 50,  200),
    ('neck',          'Tour de cou',        'cm', 'circumference', NULL, 1, 24, 1, 20,  80),
    ('bicep_right',   'Biceps droit',       'cm', 'circumference', NULL, 1, 25, 1, 15,  80),
    ('bicep_left',    'Biceps gauche',      'cm', 'circumference', NULL, 1, 26, 1, 15,  80),
    ('thigh_right',   'Cuisse droite',      'cm', 'circumference', NULL, 1, 27, 1, 20,  120),
    ('thigh_left',    'Cuisse gauche',      'cm', 'circumference', NULL, 1, 28, 1, 20,  120),
    ('calf_right',    'Mollet droit',       'cm', 'circumference', NULL, 1, 29, 1, 15,  80),
    ('calf_left',     'Mollet gauche',      'cm', 'circumference', NULL, 1, 30, 1, 15,  80);

-- 2c. body_scan (11 entries, Withings Body Scan)
INSERT OR IGNORE INTO metric_definitions (slug, label, unit, category, withings_type, is_builtin, sort_order, decimals, min_value, max_value) VALUES
    ('vo2max',              'VO2 Max',                'ml/kg/min', 'body_scan', 123, 1, 40, 1, 10,  100),
    ('visceral_fat',        'Graisse viscerale',      '',          'body_scan', 170, 1, 41, 1, 0,   30),
    ('vascular_age',        'Age vasculaire',         'annees',    'body_scan', 155, 1, 42, 0, 10,  100),
    ('pulse_wave_velocity', 'Vitesse onde de pouls',  'm/s',       'body_scan', 91,  1, 43, 2, 3,   20),
    ('bmr',                 'Metabolisme basal',      'kcal',      'body_scan', 226, 1, 44, 0, 500, 5000),
    ('extracellular_water', 'Eau extracellulaire',    'kg',        'body_scan', 168, 1, 45, 2, 1,   50),
    ('intracellular_water', 'Eau intracellulaire',    'kg',        'body_scan', 169, 1, 46, 2, 1,   50),
    ('nerve_health',        'Sante nerveuse',         '',          'body_scan', 167, 1, 47, 2, 0,   100),
    ('spo2',                'SpO2',                   '%',         'body_scan', 54,  1, 48, 1, 70,  100),
    ('body_temperature',    'Temperature corporelle', 'degC',      'body_scan', 71,  1, 49, 1, 30,  45),
    ('skin_temperature',    'Temperature cutanee',    'degC',      'body_scan', 73,  1, 50, 1, 20,  45);

-- 2d. derived (6 entries, calcules)
INSERT OR IGNORE INTO metric_definitions (slug, label, unit, category, withings_type, is_builtin, sort_order, decimals, min_value, max_value) VALUES
    ('bmi_calculated',    'IMC (calcule)',             'kg/m2', 'derived', NULL, 1, 60, 2, 10,  60),
    ('ffmi',              'FFMI',                      'kg/m2', 'derived', NULL, 1, 61, 2, 10,  40),
    ('waist_hip_ratio',   'Ratio taille/hanches',      '',      'derived', NULL, 1, 62, 3, 0.3, 2),
    ('fat_free_pct',      'Masse maigre (%)',          '%',     'derived', NULL, 1, 63, 2, 30,  99),
    ('water_pct',         'Hydratation (%)',           '%',     'derived', NULL, 1, 64, 2, 20,  90),
    ('muscle_fat_ratio',  'Ratio muscle/masse grasse', '',      'derived', NULL, 1, 65, 2, 0,   50);

-- 3. Index sur tables _legacy (documentation)
-- Ces index ne sont PAS executes ici : SQLite echoue sur CREATE INDEX si la table n'existe pas
-- (DB fraiche = pas de _legacy). Les CREATE INDEX sont executes en Rust dans ensure_critical_tables (fonction retiree depuis, le schema vient des seules migrations),
-- juste apres les ALTER TABLE ... RENAME (Task 9).

-- CREATE INDEX IF NOT EXISTS idx_legacy_body_measures_date ON body_measures_legacy(measured_at)
-- CREATE INDEX IF NOT EXISTS idx_legacy_manual_date ON manual_measurements_legacy(measured_at)
-- CREATE INDEX IF NOT EXISTS idx_legacy_manual_type ON manual_measurements_legacy(measurement_type_id)
-- CREATE INDEX IF NOT EXISTS idx_legacy_vitals_date ON vitals_legacy(measured_at)
-- CREATE INDEX IF NOT EXISTS idx_legacy_vitals_type ON vitals_legacy(type)
