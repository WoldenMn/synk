-- Migration 022 : appareil d'origine d'une mesure (R39).
--
-- Un import Apple Sante range sous la source apple_health des mesures que
-- Withings, Oura ou une autre app ont ecrites dans Apple Sante. La colonne
-- device garde le nom de l'app ou de l'appareil d'origine. Colonne ajoutee
-- sans valeur par defaut : les lignes existantes restent a NULL, rien n'est
-- recopie ni reconstruit.

ALTER TABLE body_measures_legacy ADD COLUMN device TEXT;
ALTER TABLE vitals_legacy ADD COLUMN device TEXT;
ALTER TABLE blood_pressure ADD COLUMN device TEXT;
ALTER TABLE sleep_records ADD COLUMN device TEXT;
ALTER TABLE activity ADD COLUMN device TEXT;
ALTER TABLE workouts ADD COLUMN device TEXT;
ALTER TABLE metrics ADD COLUMN device TEXT;
