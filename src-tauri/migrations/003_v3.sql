-- =============================================
-- Synk V3 migration
-- =============================================

-- Measurement type goals
ALTER TABLE measurement_types ADD COLUMN target_value REAL;
ALTER TABLE measurement_types ADD COLUMN target_direction TEXT;

-- Recreate vitals with expanded CHECK (Body Scan types)
DROP TABLE IF EXISTS vitals_v2;
CREATE TABLE vitals_v2 (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    measured_at TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings' CHECK (source IN ('withings', 'manual')),
    withings_grpid TEXT,
    vital_type TEXT NOT NULL CHECK (vital_type IN (
        'temperature', 'body_temperature', 'skin_temperature', 'spo2',
        'vo2max', 'vascular_age', 'pulse_wave_velocity',
        'visceral_fat', 'bmr', 'extracellular_water', 'intracellular_water'
    )),
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
