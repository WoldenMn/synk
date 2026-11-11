-- Migration 013 : rendre la temperature (Withings measure type 12) visible.
-- vital_type_name(12)='temperature' ecrit deja dans vitals_legacy. Il manquait
-- la definition metrique (categorie body_scan, deja autorisee par le CHECK 009).
INSERT OR IGNORE INTO metric_definitions
    (slug, label, unit, category, withings_type, is_builtin, sort_order, decimals, min_value, max_value)
VALUES
    ('temperature', 'Temperature', '°C', 'body_scan', 12, 1, 81, 1, 20, 45);
