-- Migration 020 : cle etrangere sur sources et external_id (R33, seconde moitie).
--
-- Les CHECK (source IN ...) des tables de mesures sont remplaces par une cle
-- etrangere vers sources, et chaque table recoit external_id avec
-- UNIQUE(source, external_id). Reconstruction par le patron officiel de SQLite
-- (PRAGMA foreign_keys hors transaction). Les colonnes withings_grpid,
-- withings_id et withings_ecg_id restent. Rien n'est supprime : chaque ligne
-- est recopiee avec son identifiant, la sequence AUTOINCREMENT est reprise.
-- Les noms _legacy sont supposes : le demarrage les pose avant la 019.

PRAGMA foreign_keys=OFF;

DROP VIEW IF EXISTS body_measures;
DROP VIEW IF EXISTS vitals;

CREATE TABLE body_measures_legacy_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    measured_at TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual' REFERENCES sources(id),
    withings_grpid TEXT UNIQUE,
    weight REAL,
    fat_ratio REAL,
    fat_mass REAL,
    fat_free_mass REAL,
    muscle_mass REAL,
    bone_mass REAL,
    hydration REAL,
    bmi REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    external_id TEXT,
    UNIQUE (source, external_id)
);
INSERT INTO body_measures_legacy_new (id, measured_at, source, withings_grpid, weight, fat_ratio, fat_mass, fat_free_mass, muscle_mass, bone_mass, hydration, bmi, created_at)
    SELECT id, measured_at, source, withings_grpid, weight, fat_ratio, fat_mass, fat_free_mass, muscle_mass, bone_mass, hydration, bmi, created_at FROM body_measures_legacy;
DELETE FROM sqlite_sequence WHERE name = 'body_measures_legacy_new';
INSERT INTO sqlite_sequence (name, seq) SELECT 'body_measures_legacy_new', seq FROM sqlite_sequence WHERE name = 'body_measures_legacy';
DROP TABLE body_measures_legacy;
ALTER TABLE body_measures_legacy_new RENAME TO body_measures_legacy;
CREATE INDEX idx_body_measures_date ON body_measures_legacy(measured_at);
CREATE VIEW body_measures AS SELECT * FROM body_measures_legacy;

CREATE TABLE vitals_legacy_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    measured_at TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings' REFERENCES sources(id),
    withings_grpid TEXT,
    vital_type TEXT NOT NULL,
    value REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    external_id TEXT,
    UNIQUE (withings_grpid, vital_type),
    UNIQUE (source, external_id)
);
INSERT INTO vitals_legacy_new (id, measured_at, source, withings_grpid, vital_type, value, created_at)
    SELECT id, measured_at, source, withings_grpid, vital_type, value, created_at FROM vitals_legacy;
DELETE FROM sqlite_sequence WHERE name = 'vitals_legacy_new';
INSERT INTO sqlite_sequence (name, seq) SELECT 'vitals_legacy_new', seq FROM sqlite_sequence WHERE name = 'vitals_legacy';
DROP TABLE vitals_legacy;
ALTER TABLE vitals_legacy_new RENAME TO vitals_legacy;
CREATE INDEX idx_vitals_date ON vitals_legacy(measured_at);
CREATE INDEX idx_vitals_type ON vitals_legacy(vital_type);
CREATE VIEW vitals AS SELECT * FROM vitals_legacy;

CREATE TABLE blood_pressure_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    measured_at TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual' REFERENCES sources(id),
    withings_grpid TEXT UNIQUE,
    systolic INTEGER NOT NULL,
    diastolic INTEGER NOT NULL,
    heart_rate INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    external_id TEXT,
    UNIQUE (source, external_id)
);
INSERT INTO blood_pressure_new (id, measured_at, source, withings_grpid, systolic, diastolic, heart_rate, created_at)
    SELECT id, measured_at, source, withings_grpid, systolic, diastolic, heart_rate, created_at FROM blood_pressure;
DELETE FROM sqlite_sequence WHERE name = 'blood_pressure_new';
INSERT INTO sqlite_sequence (name, seq) SELECT 'blood_pressure_new', seq FROM sqlite_sequence WHERE name = 'blood_pressure';
DROP TABLE blood_pressure;
ALTER TABLE blood_pressure_new RENAME TO blood_pressure;
CREATE INDEX idx_blood_pressure_date ON blood_pressure(measured_at);

CREATE TABLE sleep_records_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings' REFERENCES sources(id),
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
    sleep_efficiency REAL,
    nb_rem_episodes INTEGER,
    out_of_bed_count INTEGER,
    waso INTEGER,
    total_timeinbed INTEGER,
    apnea_hypopnea_index REAL,
    breathing_disturbances_intensity REAL,
    asleepduration INTEGER,
    night_events INTEGER,
    external_id TEXT,
    UNIQUE (date, source),
    UNIQUE (source, external_id)
);
INSERT INTO sleep_records_new (id, date, source, withings_id, start_at, end_at, duration_total, duration_deep, duration_light, duration_rem, duration_awake, duration_to_sleep, duration_to_wakeup, sleep_score, hr_average, hr_min, hr_max, rr_average, snoring, snoring_count, created_at, sleep_efficiency, nb_rem_episodes, out_of_bed_count, waso, total_timeinbed, apnea_hypopnea_index, breathing_disturbances_intensity, asleepduration, night_events)
    SELECT id, date, source, withings_id, start_at, end_at, duration_total, duration_deep, duration_light, duration_rem, duration_awake, duration_to_sleep, duration_to_wakeup, sleep_score, hr_average, hr_min, hr_max, rr_average, snoring, snoring_count, created_at, sleep_efficiency, nb_rem_episodes, out_of_bed_count, waso, total_timeinbed, apnea_hypopnea_index, breathing_disturbances_intensity, asleepduration, night_events FROM sleep_records;
DELETE FROM sqlite_sequence WHERE name = 'sleep_records_new';
INSERT INTO sqlite_sequence (name, seq) SELECT 'sleep_records_new', seq FROM sqlite_sequence WHERE name = 'sleep_records';
DROP TABLE sleep_records;
ALTER TABLE sleep_records_new RENAME TO sleep_records;
CREATE INDEX idx_sleep_date ON sleep_records(date);

CREATE TABLE activity_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings' REFERENCES sources(id),
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
    hr_zone_0 INTEGER,
    hr_zone_1 INTEGER,
    hr_zone_2 INTEGER,
    hr_zone_3 INTEGER,
    external_id TEXT,
    UNIQUE (date, source),
    UNIQUE (source, external_id)
);
INSERT INTO activity_new (id, date, source, steps, distance, elevation, calories_active, calories_total, duration_soft, duration_moderate, duration_intense, duration_active, hr_average, hr_min, hr_max, created_at, hr_zone_0, hr_zone_1, hr_zone_2, hr_zone_3)
    SELECT id, date, source, steps, distance, elevation, calories_active, calories_total, duration_soft, duration_moderate, duration_intense, duration_active, hr_average, hr_min, hr_max, created_at, hr_zone_0, hr_zone_1, hr_zone_2, hr_zone_3 FROM activity;
DELETE FROM sqlite_sequence WHERE name = 'activity_new';
INSERT INTO sqlite_sequence (name, seq) SELECT 'activity_new', seq FROM sqlite_sequence WHERE name = 'activity';
DROP TABLE activity;
ALTER TABLE activity_new RENAME TO activity;
CREATE INDEX idx_activity_date ON activity(date);

CREATE TABLE workouts_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    start_at TEXT,
    end_at TEXT,
    source TEXT NOT NULL DEFAULT 'withings' REFERENCES sources(id),
    withings_id TEXT,
    category INTEGER,
    category_label TEXT,
    duration INTEGER,
    pause_duration INTEGER,
    distance REAL,
    steps INTEGER,
    elevation REAL,
    calories REAL,
    hr_average INTEGER,
    hr_min INTEGER,
    hr_max INTEGER,
    hr_zone_0 INTEGER,
    hr_zone_1 INTEGER,
    hr_zone_2 INTEGER,
    hr_zone_3 INTEGER,
    spo2_average REAL,
    intensity INTEGER,
    effort_score REAL,
    pool_laps INTEGER,
    pool_length INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    external_id TEXT,
    UNIQUE (date, withings_id),
    UNIQUE (source, external_id)
);
INSERT INTO workouts_new (id, date, start_at, end_at, source, withings_id, category, category_label, duration, pause_duration, distance, steps, elevation, calories, hr_average, hr_min, hr_max, hr_zone_0, hr_zone_1, hr_zone_2, hr_zone_3, spo2_average, intensity, effort_score, pool_laps, pool_length, created_at)
    SELECT id, date, start_at, end_at, source, withings_id, category, category_label, duration, pause_duration, distance, steps, elevation, calories, hr_average, hr_min, hr_max, hr_zone_0, hr_zone_1, hr_zone_2, hr_zone_3, spo2_average, intensity, effort_score, pool_laps, pool_length, created_at FROM workouts;
DELETE FROM sqlite_sequence WHERE name = 'workouts_new';
INSERT INTO sqlite_sequence (name, seq) SELECT 'workouts_new', seq FROM sqlite_sequence WHERE name = 'workouts';
DROP TABLE workouts;
ALTER TABLE workouts_new RENAME TO workouts;
CREATE INDEX idx_workouts_date ON workouts(date);

CREATE TABLE heart_records_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings' REFERENCES sources(id),
    withings_ecg_id TEXT,
    ecg_status INTEGER,
    heart_rate INTEGER,
    qrs_duration INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    external_id TEXT,
    UNIQUE (timestamp, source),
    UNIQUE (source, external_id)
);
INSERT INTO heart_records_new (id, timestamp, source, withings_ecg_id, ecg_status, heart_rate, qrs_duration, created_at)
    SELECT id, timestamp, source, withings_ecg_id, ecg_status, heart_rate, qrs_duration, created_at FROM heart_records;
DELETE FROM sqlite_sequence WHERE name = 'heart_records_new';
INSERT INTO sqlite_sequence (name, seq) SELECT 'heart_records_new', seq FROM sqlite_sequence WHERE name = 'heart_records';
DROP TABLE heart_records;
ALTER TABLE heart_records_new RENAME TO heart_records;
CREATE INDEX idx_heart_timestamp ON heart_records(timestamp);

CREATE TABLE metrics_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    definition_id INTEGER NOT NULL REFERENCES metric_definitions(id) ON DELETE CASCADE,
    measured_at TEXT NOT NULL,
    value REAL NOT NULL,
    source TEXT NOT NULL REFERENCES sources(id),
    external_id TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (definition_id, measured_at, source, external_id)
);
INSERT INTO metrics_new (id, definition_id, measured_at, value, source, external_id, notes, created_at)
    SELECT id, definition_id, measured_at, value, source, external_id, notes, created_at FROM metrics;
DELETE FROM sqlite_sequence WHERE name = 'metrics_new';
INSERT INTO sqlite_sequence (name, seq) SELECT 'metrics_new', seq FROM sqlite_sequence WHERE name = 'metrics';
DROP TABLE metrics;
ALTER TABLE metrics_new RENAME TO metrics;
CREATE INDEX idx_metrics_definition_date ON metrics(definition_id, measured_at);
CREATE INDEX idx_metrics_external ON metrics(external_id) WHERE external_id IS NOT NULL;
CREATE INDEX idx_metrics_source ON metrics(source);

PRAGMA foreign_keys=ON;
