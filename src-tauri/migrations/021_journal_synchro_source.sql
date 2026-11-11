-- Migration 021 : le journal de synchronisation nomme sa source (R34).
--
-- Une ligne de sync_log decrit une etape (mesures, sommeil...) d'une source.
-- Jusqu'ici une seule source existait : les lignes deja ecrites sont celles de
-- Withings, d'ou la valeur par defaut. Pas de cle etrangere : SQLite refuse
-- d'ajouter une colonne REFERENCES avec une valeur par defaut non nulle, et
-- le journal est technique, une source retiree du registre ne doit pas
-- bloquer l'ecriture de sa derniere ligne.

ALTER TABLE sync_log ADD COLUMN source TEXT NOT NULL DEFAULT 'withings';
CREATE INDEX idx_sync_log_source ON sync_log(source, data_type);
