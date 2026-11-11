-- =============================================
-- Synk migration 006 : Consolidation des tables auparavant
-- creees uniquement dans ensure_critical_tables (fonction retiree depuis, le schema vient des seules migrations).
-- =============================================
-- Ces tables n'existaient dans aucune migration versionnee.
-- Elles sont desormais crees ici pour que les migrations
-- soient la seule source de verite du schema.

CREATE TABLE IF NOT EXISTS user_profile (
    id INTEGER PRIMARY KEY,
    age INTEGER,
    sex TEXT,
    height_cm REAL
);

CREATE TABLE IF NOT EXISTS health_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_key TEXT NOT NULL,
    condition TEXT NOT NULL,
    threshold REAL NOT NULL,
    label TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS alert_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    alert_id INTEGER REFERENCES health_alerts(id),
    value REAL NOT NULL,
    triggered_at TEXT NOT NULL,
    seen INTEGER DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_alert_events_alert ON alert_events(alert_id);
CREATE INDEX IF NOT EXISTS idx_alert_events_date ON alert_events(triggered_at);

CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action TEXT NOT NULL,
    entity TEXT NOT NULL,
    entity_id INTEGER,
    details TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_audit_log_date ON audit_log(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity);

CREATE TABLE IF NOT EXISTS medications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    dosage TEXT,
    frequency TEXT,
    start_date TEXT,
    end_date TEXT,
    notes TEXT,
    active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS medication_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    medication_id INTEGER REFERENCES medications(id),
    taken_at TEXT NOT NULL,
    skipped INTEGER DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_medication_logs_med ON medication_logs(medication_id);
CREATE INDEX IF NOT EXISTS idx_medication_logs_date ON medication_logs(taken_at);

CREATE TABLE IF NOT EXISTS mood_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL UNIQUE,
    score INTEGER NOT NULL CHECK(score >= 1 AND score <= 10),
    energy INTEGER NOT NULL CHECK(energy >= 1 AND energy <= 10),
    stress INTEGER NOT NULL CHECK(stress >= 1 AND stress <= 10),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_mood_logs_date ON mood_logs(date);

CREATE TABLE IF NOT EXISTS nutrition_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    calories INTEGER,
    protein REAL,
    carbs REAL,
    fat REAL,
    water_ml INTEGER,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_nutrition_logs_date ON nutrition_logs(date);

CREATE TABLE IF NOT EXISTS lab_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    test_name TEXT NOT NULL,
    value REAL NOT NULL,
    unit TEXT,
    reference_min REAL,
    reference_max REAL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_lab_results_date ON lab_results(date);
CREATE INDEX IF NOT EXISTS idx_lab_results_test ON lab_results(test_name);

CREATE TABLE IF NOT EXISTS sync_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    data_type TEXT NOT NULL,
    duration_ms INTEGER,
    records_added INTEGER DEFAULT 0,
    status TEXT NOT NULL,
    error TEXT
);
CREATE INDEX IF NOT EXISTS idx_sync_log_date ON sync_log(started_at);
CREATE INDEX IF NOT EXISTS idx_sync_log_type ON sync_log(data_type);

CREATE TABLE IF NOT EXISTS journal_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    tags TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_journal_date ON journal_entries(date);

CREATE TABLE IF NOT EXISTS insights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL,
    severity TEXT NOT NULL,
    metric TEXT,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    value REAL,
    delta REAL,
    generated_at TEXT NOT NULL DEFAULT (datetime('now')),
    seen INTEGER DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_insights_seen ON insights(seen);
CREATE INDEX IF NOT EXISTS idx_insights_date ON insights(generated_at);

CREATE TABLE IF NOT EXISTS dashboard_layout (
    id INTEGER PRIMARY KEY,
    layout TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
