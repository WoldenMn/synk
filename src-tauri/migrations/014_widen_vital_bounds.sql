-- Migration 014 : elargir les bornes de validation des vitaux cliniques.
-- Les bornes physiologiques normales rejetaient les valeurs pathologiques reelles
-- (SpO2 desaturation, hypothermie). On passe a des bornes physiquement possibles.
-- min_value/max_value ne servent qu'a la validation d'ecriture (sanitize_measure).
UPDATE metric_definitions SET min_value = 0, max_value = 100 WHERE slug = 'spo2';
UPDATE metric_definitions SET min_value = 0, max_value = 60 WHERE slug = 'body_temperature';
UPDATE metric_definitions SET min_value = 0, max_value = 60 WHERE slug = 'temperature';
