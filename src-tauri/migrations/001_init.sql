CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    theme TEXT NOT NULL DEFAULT 'dark' CHECK (theme IN ('dark', 'light')),
    language TEXT NOT NULL DEFAULT 'fr'
);

CREATE TABLE IF NOT EXISTS withings_auth (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    token_type TEXT NOT NULL DEFAULT 'Bearer',
    withings_user_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sync_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    data_type TEXT NOT NULL UNIQUE CHECK (data_type IN ('measures', 'activity', 'sleep', 'heart')),
    last_sync_at INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS body_measures (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    measured_at TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual' CHECK (source IN ('withings', 'manual')),
    withings_grpid TEXT UNIQUE,
    weight REAL,
    fat_ratio REAL,
    fat_mass REAL,
    fat_free_mass REAL,
    muscle_mass REAL,
    bone_mass REAL,
    hydration REAL,
    bmi REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_body_measures_date ON body_measures(measured_at);

CREATE TABLE IF NOT EXISTS blood_pressure (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    measured_at TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual' CHECK (source IN ('withings', 'manual')),
    withings_grpid TEXT UNIQUE,
    systolic INTEGER NOT NULL,
    diastolic INTEGER NOT NULL,
    heart_rate INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_blood_pressure_date ON blood_pressure(measured_at);

CREATE TABLE IF NOT EXISTS sleep_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings' CHECK (source IN ('withings', 'manual')),
    withings_id TEXT,
    start_at TEXT,
    end_at TEXT,
    duration_total INTEGER,
    duration_deep INTEGER,
    duration_light INTEGER,
    duration_rem INTEGER,
    duration_awake INTEGER,
    duration_to_sleep INTEGER,
    duration_to_wakeup INTEGER,
    sleep_score INTEGER,
    hr_average INTEGER,
    hr_min INTEGER,
    hr_max INTEGER,
    rr_average REAL,
    snoring INTEGER,
    snoring_count INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(date, source)
);
CREATE INDEX IF NOT EXISTS idx_sleep_date ON sleep_records(date);

CREATE TABLE IF NOT EXISTS activity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings' CHECK (source IN ('withings', 'manual')),
    steps INTEGER,
    distance REAL,
    elevation REAL,
    calories_active REAL,
    calories_total REAL,
    duration_soft INTEGER,
    duration_moderate INTEGER,
    duration_intense INTEGER,
    duration_active INTEGER,
    hr_average INTEGER,
    hr_min INTEGER,
    hr_max INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(date, source)
);
CREATE INDEX IF NOT EXISTS idx_activity_date ON activity(date);

CREATE TABLE IF NOT EXISTS vitals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    measured_at TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings' CHECK (source IN ('withings', 'manual')),
    withings_grpid TEXT,
    vital_type TEXT NOT NULL CHECK (vital_type IN ('temperature', 'body_temperature', 'skin_temperature', 'spo2')),
    value REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(withings_grpid, vital_type)
);
CREATE INDEX IF NOT EXISTS idx_vitals_date ON vitals(measured_at);
CREATE INDEX IF NOT EXISTS idx_vitals_type ON vitals(vital_type);

CREATE TABLE IF NOT EXISTS measurement_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    unit TEXT NOT NULL DEFAULT 'cm',
    category TEXT NOT NULL DEFAULT 'circumference',
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS manual_measurements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    measurement_type_id INTEGER NOT NULL REFERENCES measurement_types(id) ON DELETE CASCADE,
    measured_at TEXT NOT NULL,
    value REAL NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_manual_measurements_date ON manual_measurements(measured_at);
CREATE INDEX IF NOT EXISTS idx_manual_measurements_type ON manual_measurements(measurement_type_id);

CREATE TABLE IF NOT EXISTS goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_key TEXT NOT NULL UNIQUE,
    target_value REAL NOT NULL,
    direction TEXT NOT NULL DEFAULT 'decrease' CHECK (direction IN ('decrease', 'increase', 'maintain')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Seed data: default settings
INSERT OR IGNORE INTO settings (id, theme, language) VALUES (1, 'dark', 'fr');

-- Seed data: sync state
INSERT OR IGNORE INTO sync_state (data_type, last_sync_at) VALUES ('measures', 0);
INSERT OR IGNORE INTO sync_state (data_type, last_sync_at) VALUES ('activity', 0);
INSERT OR IGNORE INTO sync_state (data_type, last_sync_at) VALUES ('sleep', 0);
INSERT OR IGNORE INTO sync_state (data_type, last_sync_at) VALUES ('heart', 0);

-- Seed data: built-in measurement types
INSERT OR IGNORE INTO measurement_types (slug, label, unit, category, sort_order, is_builtin) VALUES
    ('waist', 'Tour de taille', 'cm', 'circumference', 1, 1),
    ('chest', 'Tour de poitrine', 'cm', 'circumference', 2, 1),
    ('hips', 'Tour de hanches', 'cm', 'circumference', 3, 1),
    ('shoulders', 'Tour d''epaules', 'cm', 'circumference', 4, 1),
    ('neck', 'Tour de cou', 'cm', 'circumference', 5, 1),
    ('bicep_right', 'Biceps droit', 'cm', 'circumference', 6, 1),
    ('bicep_left', 'Biceps gauche', 'cm', 'circumference', 7, 1),
    ('thigh_right', 'Cuisse droite', 'cm', 'circumference', 8, 1),
    ('thigh_left', 'Cuisse gauche', 'cm', 'circumference', 9, 1),
    ('calf_right', 'Mollet droit', 'cm', 'circumference', 10, 1),
    ('calf_left', 'Mollet gauche', 'cm', 'circumference', 11, 1);
