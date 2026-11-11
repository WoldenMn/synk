PRAGMA foreign_keys=OFF;
DROP INDEX IF EXISTS idx_legacy_body_measures_date;
DROP INDEX IF EXISTS idx_legacy_manual_type;
DROP INDEX IF EXISTS idx_legacy_manual_date;
DROP INDEX IF EXISTS idx_legacy_vitals_type;
DROP INDEX IF EXISTS idx_legacy_vitals_date;
DROP INDEX IF EXISTS idx_journal_date;
DROP INDEX IF EXISTS idx_mood_logs_date;
CREATE TABLE alert_events_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    alert_id INTEGER REFERENCES health_alerts(id) ON DELETE CASCADE,
    value REAL NOT NULL,
    triggered_at TEXT NOT NULL,
    seen INTEGER DEFAULT 0
);
INSERT INTO alert_events_new (id, alert_id, value, triggered_at, seen)
    SELECT id, alert_id, value, triggered_at, seen FROM alert_events;
DROP TABLE alert_events;
ALTER TABLE alert_events_new RENAME TO alert_events;
CREATE INDEX IF NOT EXISTS idx_alert_events_alert ON alert_events(alert_id);
CREATE INDEX IF NOT EXISTS idx_alert_events_date ON alert_events(triggered_at);
CREATE TABLE medication_logs_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    medication_id INTEGER REFERENCES medications(id) ON DELETE CASCADE,
    taken_at TEXT NOT NULL,
    skipped INTEGER DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO medication_logs_new (id, medication_id, taken_at, skipped, notes, created_at)
    SELECT id, medication_id, taken_at, skipped, notes, created_at FROM medication_logs;
DROP TABLE medication_logs;
ALTER TABLE medication_logs_new RENAME TO medication_logs;
CREATE INDEX IF NOT EXISTS idx_medication_logs_med ON medication_logs(medication_id);
CREATE INDEX IF NOT EXISTS idx_medication_logs_date ON medication_logs(taken_at);
PRAGMA foreign_keys=ON;
