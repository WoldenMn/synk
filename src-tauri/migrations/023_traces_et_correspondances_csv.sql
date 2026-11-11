-- Migration 023 : traces GPX et TCX, correspondances CSV (R40).
--
-- Une source fichier de plus, trace, pour les fichiers GPX et TCX : un
-- entrainement GPX n'est pas un fichier FIT, l'etiquette de source doit dire
-- la verite. Une table de correspondances CSV garde, par signature d'en-tete,
-- le reglage choisi dans l'assistant pour le reutiliser a l'import suivant.
-- Rien n'est recopie ni reconstruit.

INSERT OR IGNORE INTO sources (id, label, kind, sort_order)
    VALUES ('trace', 'Traces GPX et TCX', 'file', 105);

CREATE TABLE csv_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    signature TEXT NOT NULL,
    config TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_csv_mappings_signature ON csv_mappings(signature);
