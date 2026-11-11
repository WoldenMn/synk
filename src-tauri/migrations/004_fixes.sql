-- V4: Fix heart_records UNIQUE constraint + add nerve_health vital type

-- Recreate heart_records with proper UNIQUE on (timestamp, source) instead of (withings_ecg_id)
DROP TABLE IF EXISTS heart_records_v2;
CREATE TABLE heart_records_v2 (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings' CHECK (source IN ('withings', 'manual')),
    withings_ecg_id TEXT,
    ecg_status INTEGER,
    heart_rate INTEGER,
    qrs_duration INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(timestamp, source)
);
INSERT OR IGNORE INTO heart_records_v2 (id, timestamp, source, withings_ecg_id, ecg_status, heart_rate, qrs_duration, created_at)
    SELECT id, timestamp, source, withings_ecg_id, ecg_status, heart_rate, qrs_duration, created_at FROM heart_records;
DROP TABLE IF EXISTS heart_records;
ALTER TABLE heart_records_v2 RENAME TO heart_records;
CREATE INDEX IF NOT EXISTS idx_heart_timestamp ON heart_records(timestamp);

-- Recreate vitals with nerve_health added to CHECK
DROP TABLE IF EXISTS vitals_v2;
CREATE TABLE vitals_v2 (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    measured_at TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings' CHECK (source IN ('withings', 'manual')),
    withings_grpid TEXT,
    vital_type TEXT NOT NULL CHECK (vital_type IN ('temperature', 'body_temperature', 'skin_temperature', 'spo2', 'vo2max', 'vascular_age', 'pulse_wave_velocity', 'nerve_health', 'visceral_fat', 'bmr', 'extracellular_water', 'intracellular_water')),
    value REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(withings_grpid, vital_type)
);
INSERT OR IGNORE INTO vitals_v2 (id, measured_at, source, withings_grpid, vital_type, value, created_at)
    SELECT id, measured_at, source, withings_grpid, vital_type, value, created_at FROM vitals;
DROP TABLE IF EXISTS vitals;
ALTER TABLE vitals_v2 RENAME TO vitals;
CREATE INDEX IF NOT EXISTS idx_vitals_date ON vitals(measured_at);
CREATE INDEX IF NOT EXISTS idx_vitals_type ON vitals(vital_type);
