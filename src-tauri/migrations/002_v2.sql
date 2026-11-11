-- =============================================
-- Synk V2 migration
-- =============================================

-- New table: workouts
CREATE TABLE IF NOT EXISTS workouts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    start_at TEXT,
    end_at TEXT,
    source TEXT NOT NULL DEFAULT 'withings' CHECK (source IN ('withings', 'manual')),
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
    UNIQUE(date, withings_id)
);
CREATE INDEX IF NOT EXISTS idx_workouts_date ON workouts(date);

-- New table: heart_records
CREATE TABLE IF NOT EXISTS heart_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings',
    withings_ecg_id TEXT UNIQUE,
    ecg_status INTEGER,
    heart_rate INTEGER,
    qrs_duration INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_heart_timestamp ON heart_records(timestamp);

-- New table: withings_devices
CREATE TABLE IF NOT EXISTS withings_devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL UNIQUE,
    device_type TEXT,
    model TEXT,
    model_id INTEGER,
    battery TEXT,
    last_session_date TEXT,
    first_session_date TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Sleep: 9 new columns
ALTER TABLE sleep_records ADD COLUMN sleep_efficiency REAL;
ALTER TABLE sleep_records ADD COLUMN nb_rem_episodes INTEGER;
ALTER TABLE sleep_records ADD COLUMN out_of_bed_count INTEGER;
ALTER TABLE sleep_records ADD COLUMN waso INTEGER;
ALTER TABLE sleep_records ADD COLUMN total_timeinbed INTEGER;
ALTER TABLE sleep_records ADD COLUMN apnea_hypopnea_index REAL;
ALTER TABLE sleep_records ADD COLUMN breathing_disturbances_intensity REAL;
ALTER TABLE sleep_records ADD COLUMN asleepduration INTEGER;
ALTER TABLE sleep_records ADD COLUMN night_events INTEGER;

-- Activity: 4 HR zone columns
ALTER TABLE activity ADD COLUMN hr_zone_0 INTEGER;
ALTER TABLE activity ADD COLUMN hr_zone_1 INTEGER;
ALTER TABLE activity ADD COLUMN hr_zone_2 INTEGER;
ALTER TABLE activity ADD COLUMN hr_zone_3 INTEGER;

-- Recreate sync_state with expanded CHECK constraint
DROP TABLE IF EXISTS sync_state_v2;
CREATE TABLE sync_state_v2 (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    data_type TEXT NOT NULL UNIQUE CHECK (data_type IN ('measures', 'activity', 'sleep', 'heart', 'workouts', 'devices')),
    last_sync_at INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT OR IGNORE INTO sync_state_v2 (data_type, last_sync_at, updated_at)
    SELECT data_type, last_sync_at, updated_at FROM sync_state;
DROP TABLE IF EXISTS sync_state;
ALTER TABLE sync_state_v2 RENAME TO sync_state;
INSERT OR IGNORE INTO sync_state (data_type, last_sync_at) VALUES ('workouts', 0);
INSERT OR IGNORE INTO sync_state (data_type, last_sync_at) VALUES ('devices', 0);

-- Recreate vitals with expanded CHECK constraint
DROP TABLE IF EXISTS vitals_v2;
CREATE TABLE vitals_v2 (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    measured_at TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'withings' CHECK (source IN ('withings', 'manual')),
    withings_grpid TEXT,
    vital_type TEXT NOT NULL CHECK (vital_type IN (
        'temperature', 'body_temperature', 'skin_temperature', 'spo2',
        'vo2max', 'vascular_age', 'pulse_wave_velocity'
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
