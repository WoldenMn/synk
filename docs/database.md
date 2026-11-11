# Base de données

Le schéma SQLite de Synk, son système de migration et ses conventions, d'après `src-tauri/migrations/` et `src-tauri/src/db/`. Une base neuve, une fois le registre appliqué, compte 43 tables/vues : 36 tables et 7 vues.

## 1. Vue d'ensemble

| Table ou vue | Groupe | Rôle |
|---|---|---|
| `body_measures_legacy` (vue `body_measures`) | Mesures | Composition corporelle |
| `blood_pressure` | Mesures | Tension et pouls |
| `sleep_records` | Mesures | Sommeil, une ligne par `(date, source)` |
| `activity` | Mesures | Activité quotidienne, une ligne par `(date, source)` |
| `workouts` | Mesures | Entraînements |
| `heart_records` | Mesures | ECG et enregistrements cardiaques |
| `vitals_legacy` (vue `vitals`) | Mesures | Constantes polymorphes (`vital_type`) |
| `metric_definitions` | Métriques V2 | Catalogue des métriques (builtin, importées, personnalisées) |
| `metrics` | Métriques V2 | Mesures qui n'ont pas de table historique |
| `metric_source_priority` | Sources | Ordre de préférence des sources, par métrique |
| `sources` | Sources | Catalogue des sources, cible de la clé étrangère `source` |
| `source_auth` | Sources | Jetons OAuth chiffrés, une ligne par source |
| `source_credentials` | Sources | `client_id` et `client_secret` chiffrés, une ligne par source |
| `source_state` | Sources | Curseurs de synchro par `(source_id, domain)` |
| vues `withings_auth`, `withings_credentials`, `sync_state` | Sources | Lecture seule, sur `source_auth`, `source_credentials`, `source_state` (source `withings`) |
| `csv_mappings` | Sources | Correspondances de colonnes mémorisées par l'assistant CSV |
| `withings_devices` | Sources | Appareils Withings |
| `measurement_types_legacy` (vue `measurement_types`) | Mensurations | 47 types intégrés plus les types de l'utilisateur |
| `manual_measurements_legacy` (vue `manual_measurements`) | Mensurations | Valeurs saisies |
| `goals` | Mensurations | Objectifs par métrique |
| `clothing_sizes` | Mensurations | Tailles de vêtements (saisies, forcées ou estimées) |
| `health_alerts`, `alert_events` | Alertes | Règles et déclenchements |
| `medications`, `medication_logs` | Suivi | Traitements et prises |
| `lab_results` | Suivi | Analyses de laboratoire |
| `journal_entries` | Suivi | Notes datées, une par jour |
| `insights` | Suivi | Constats automatiques |
| `settings`, `user_profile` | Système | Réglages (une ligne) et profil (âge, sexe, taille) |
| `sync_log` | Système | Une ligne par étape de synchro |
| `error_log` | Système | Erreurs du front et du démarrage |
| `audit_log` | Système | Journal d'audit des actions |
| `schema_version` | Système | Migrations appliquées |
| `dashboard_layout`, `mood_logs`, `nutrition_logs` | Réservées | Aucune lecture ni écriture par le code, conservées (§ 6) |

## 2. Colonnes communes aux mesures

Depuis les migrations 019 et 020, les huit tables de mesures (`body_measures_legacy`, `vitals_legacy`, `blood_pressure`, `sleep_records`, `activity`, `workouts`, `heart_records`, `metrics`) partagent trois choses :

- `source TEXT NOT NULL REFERENCES sources(id)` : toute valeur inconnue de `sources` est refusée par la base (l'import JSON l'ignore). Valeurs : `withings`, `manual`, `derived`, `import`, `oura`, `polar`, `whoop`, `google_health`, `apple_health`, `fit`, `csv`, `trace`.
- `external_id TEXT` avec `UNIQUE (source, external_id)` : l'identifiant de la ligne chez la source. Deux `NULL` ne sont jamais égaux en SQLite ; un import fournit donc toujours son `external_id`.
- `device TEXT` (migration 022, sauf `heart_records`) : l'appareil ou l'app d'origine d'une mesure importée d'Apple Santé.

## 3. Détail des tables

Les durées sont en secondes, les distances en mètres, sauf mention contraire. Toute table de mesures a aussi `id INTEGER PRIMARY KEY AUTOINCREMENT` et `created_at TEXT DEFAULT datetime('now')`.

### Mesures

**`body_measures_legacy`** : `measured_at` (ISO, indexé), `source` (défaut `manual`), `withings_grpid` (UNIQUE), `weight` (kg), `fat_ratio` (%), `fat_mass`, `fat_free_mass`, `muscle_mass`, `bone_mass` (kg), `hydration` (kg, type Withings 77), `bmi`.

**`blood_pressure`** : `measured_at` (indexé), `source` (défaut `manual`), `withings_grpid` (UNIQUE), `systolic` et `diastolic` (mmHg, obligatoires), `heart_rate` (bpm).

**`sleep_records`** : `date` (AAAA-MM-JJ, indexé), `source` (défaut `withings`), `withings_id`, `start_at`, `end_at`, `duration_total`, `duration_deep`, `duration_light`, `duration_rem`, `duration_awake`, `duration_to_sleep`, `duration_to_wakeup`, `sleep_score` (0 à 100), `hr_average`, `hr_min`, `hr_max`, `rr_average`, `snoring`, `snoring_count`, `sleep_efficiency`, `nb_rem_episodes`, `out_of_bed_count`, `waso`, `total_timeinbed`, `apnea_hypopnea_index`, `breathing_disturbances_intensity`, `asleepduration`, `night_events`. Unicité `(date, source)`.

**`activity`** : `date` (indexé), `source` (défaut `withings`), `steps`, `distance`, `elevation`, `calories_active`, `calories_total`, `duration_soft`, `duration_moderate`, `duration_intense`, `duration_active`, `hr_average`, `hr_min`, `hr_max`, `hr_zone_0` à `hr_zone_3`. Unicité `(date, source)`.

**`workouts`** : `date` (indexé), `start_at`, `end_at`, `source`, `withings_id`, `category` (code Withings) et `category_label`, `duration`, `pause_duration`, `distance`, `steps`, `elevation`, `calories`, `hr_average`, `hr_min`, `hr_max`, `hr_zone_0` à `hr_zone_3`, `spo2_average`, `intensity`, `effort_score`, `pool_laps`, `pool_length`. Unicité `(date, withings_id)`.

**`heart_records`** : `timestamp` (indexé), `source`, `withings_ecg_id`, `ecg_status` (0 normal, 1 fibrillation, 2 non concluant), `heart_rate`, `qrs_duration` (ms). Unicité `(timestamp, source)`. Pas de colonne `device`.

**`vitals_legacy`** : `measured_at`, `source`, `withings_grpid`, `vital_type` (texte libre depuis la migration 020, plus de `CHECK`), `value`. Unicité `(withings_grpid, vital_type)`. Types écrits par Withings : `temperature`, `body_temperature`, `skin_temperature`, `spo2`, `vo2max`, `vascular_age`, `pulse_wave_velocity`, `nerve_health`, `visceral_fat`, `bmr`, `extracellular_water`, `intracellular_water` ; les autres sources y ajoutent leurs constantes (`hrv`, `readiness_score`, etc.).

### Métriques

**`metric_definitions`** : `slug` (UNIQUE), `label`, `unit`, `category` (`body_composition`, `circumference`, `body_scan`, `derived`, `custom`, `vital_bp`, `vital_sleep`, `vital_activity`), `withings_type`, `is_builtin`, `sort_order`, `decimals`, `min_value`, `max_value`. Le catalogue de base vient de la migration 005 ; les sources y ajoutent leurs définitions par `Source::metric_definitions`, semées en Rust (`seed_source_definitions`).

**`metrics`** : `definition_id`, `measured_at`, `value`, `source`, `external_id`, `notes`, `device`. Unicité `(definition_id, measured_at, source, external_id)`. Depuis R28, seul le pouls sans tension y est écrit (source `Split`) ; les lignes plus anciennes y restent.

**`metric_source_priority`** : `(definition_id, source_id)` clé primaire, `rank`, `UNIQUE (definition_id, rank)`. Vide tant que l'utilisateur ne choisit rien.

### Sources

**`sources`** : `id`, `label`, `kind` (`oauth`, `file`, `internal`), `sort_order`. Douze graines : `withings`, `manual`, `derived`, `import`, `oura`, `polar`, `whoop`, `google_health`, `apple_health`, `fit`, `csv` (019) et `trace` (023).

**`source_auth`** : `source_id` (clé primaire), `access_token` et `refresh_token` (`enc:v1:`), `expires_at` (Unix), `token_type`, `user_ref`, `updated_at`, `created_at`.

**`source_credentials`** : `source_id`, `client_id`, `client_secret` (`enc:v1:`), `updated_at`.

**`source_state`** : `source_id`, `domain`, `last_sync_at` (Unix, seul champ temporel entier avec `expires_at`), `updated_at`. Les domaines vont des types de données (`measures`, `sleep`…) à des notes (`registration`, `quota`, `refresh_expiry`).

**`csv_mappings`** : `name` (UNIQUE), `signature` (indexée, SHA-256 des noms de colonnes en minuscules), `config` (JSON), `created_at`, `updated_at`. Exclue de l'export JSON.

**`withings_devices`** : `device_id` (UNIQUE), `device_type`, `model`, `model_id`, `battery`, `last_session_date`, `first_session_date`, `updated_at`.

### Mensurations

**`measurement_types_legacy`** : `slug` (UNIQUE), `label`, `unit` (défaut `cm`), `category`, `sort_order`, `is_builtin`, `target_value`, `target_direction`. Les 47 types intégrés viennent de `catalog.json` par `seed_body_catalog` (§ 5) ; un slug pris par un type de l'utilisateur n'est pas touché.

**`manual_measurements_legacy`** : `measurement_type_id` (cascade), `measured_at`, `value`, `notes`. Pas de colonne `source`.

**`goals`** : `metric_key` (UNIQUE), `target_value`, `direction` (`decrease`, `increase`, `maintain`), `updated_at`.

**`clothing_sizes`** : `category` (UNIQUE), `value`, `source` (`override`, `manual`, `estimated`), `updated_at`. Une estimation ne remplace jamais une ligne `manual` ou `override`.

### Suivi

**`health_alerts`** (`metric_key`, `condition`, `threshold`, `label`, `enabled`) et **`alert_events`** (`alert_id` en cascade, `value`, `triggered_at` en UTC avec `Z`, `seen`). **`medications`** (`name`, `dosage`, `frequency`, `start_date`, `end_date`, `notes`, `active`) et **`medication_logs`** (`medication_id` en cascade, `taken_at`, `skipped`, `notes`). **`lab_results`** (`date`, `test_name`, `value`, `unit`, `reference_min`, `reference_max`, `notes`). **`journal_entries`** (`date` UNIQUE, `content`, `tags`, `updated_at`). **`insights`** (`kind`, `severity`, `metric`, `title`, `message`, `value`, `delta`, `generated_at`, `seen`).

### Système

**`settings`** : une ligne (`CHECK (id = 1)`) : `theme`, `language`, `audit_retention_days`, `audit_max_entries`, `audit_auto_purge_enabled`. Le thème réellement utilisé vit dans le `localStorage` du front, pas ici.

**`user_profile`** : `age`, `sex`, `height_cm`, plus neuf colonnes d'identité ajoutées par la migration 015 (`birth_date`, `handedness`, `eye_color`, `hair_color`, `blood_group`, `allergies`, `vision_correction`, `medical_history`, `vaccines`) que le code ne lit ni n'écrit plus. Une seule commande d'écriture : `update_user_profile`.

**`sync_log`** : `started_at`, `finished_at`, `data_type`, `duration_ms`, `records_added`, `status` (`success` ou `error`), `error` (tronqué à 2 000 caractères), `source`. Bornée à 1 000 lignes et 90 jours, en gardant la dernière synchro réussie de chaque couple source et type.

**`error_log`** : `occurred_at`, `source`, `code`, `message`, `context`. **`audit_log`** : `action`, `entity`, `entity_id`, `details`. **`schema_version`** : `version`, `applied_at`.

## 4. Système de migration

Au démarrage, `bootstrap_schema` (`db/bootstrap.rs`) enchaîne :

1. `001_init.sql`, instruction par instruction, à chaque lancement. Sur une base existante ses échecs sont attendus (objet déjà présent, `INSERT` ou `CREATE INDEX` sur une table devenue vue) et partent en `debug` ; tout autre échec est un `warn` (`migration_001_replay_is_benign`).
2. `schema_version`, puis le registre `MIGRATIONS` (versions 2 à 23), chaque migration seulement si sa version est absente.
3. `migrate_to_v2_schema` (renommage en `*_legacy` et vues de rétrocompatibilité), passé **avant** la première migration de version 19 ou plus, qui suppose ces noms, puis rejoué.
4. `seed_body_catalog` et `seed_source_definitions` (Rust, jamais une migration, voir § 5).

Ajouter une migration : un fichier dans `src-tauri/migrations/` et une ligne `migration!(n, "nnn_nom.sql")`. `LATEST_VERSION` et `EXPECTED_LATEST_VERSION` (`boot.rs`) en sont déduits ; `tests/integration/schema.rs` refuse un fichier non enregistré, des versions non consécutives et une base neuve qui ne finit pas à la dernière.

`run_versioned_migration` (`db/migrations.rs`) :

- Découpe le SQL sur `;`, isole les `PRAGMA` de tête et de queue (exécutés hors transaction, parce que `PRAGMA foreign_keys` n'agit pas dans une transaction) et enveloppe le reste dans `BEGIN`/`COMMIT`/`ROLLBACK`.
- Tolérance figée aux migrations 2 à 16 (`tolerated` : `duplicate column`, `already another table`, `already exists`, `no such table`, avec un `warn`). Dès la 17, aucune erreur n'est tolérée.
- Dès la 17, le nombre de lignes de `PRAGMA foreign_key_check` est compté avant et avant le `COMMIT` ; s'il augmente, la migration est annulée et le démarrage échoue.
- Un test refuse un `;` dans un commentaire ou une chaîne d'un fichier de migration, et tout `CREATE TRIGGER`.

Le motif **CREATE-INSERT-DROP-RENAME** (recréer une table pour changer un `CHECK` ou une clé étrangère) est celui des migrations 002, 003, 004, 009, 016, 017, 019 et 020.

## 5. Historique des migrations

| N° | Fichier | Apport |
|---|---|---|
| 001 | `001_init.sql` | Schéma initial : réglages, mesures, mensurations, objectifs |
| 002 | `002_v2.sql` | `workouts`, `heart_records`, `withings_devices`, colonnes de sommeil et zones de pouls |
| 003 | `003_v3.sql` | Objectifs par type de mensuration, constantes Body Scan |
| 004 | `004_fixes.sql` | Unicité de `heart_records` sur `(timestamp, source)`, `nerve_health` |
| 005 | `005_metrics_unification.sql` | `metric_definitions`, `metrics`, renommage en `*_legacy` et vues |
| 006 | `006_consolidation.sql` | Tables créées jusque-là par le code : profil, alertes, audit, médicaments, analyses, journal, constats, `sync_log` |
| 007 | `007_withings_credentials.sql` | Identifiants Withings (chiffrés depuis V2.8.0) |
| 008 | `008_audit_purge_settings.sql` | Réglages de purge de l'audit |
| 009 | `009_metric_definitions_extension.sql` | Catégories `vital_bp`, `vital_sleep`, `vital_activity` |
| 010 | `010_sync_state_metrics.sql` | Curseur `metrics` |
| 011 | `011_clothing_sizes.sql` | Tailles de vêtements |
| 012 | `012_error_log.sql` | Journal d'erreurs |
| 013 | `013_temperature_metric.sql` | Température visible dans le catalogue |
| 014 | `014_widen_vital_bounds.sql` | Bornes cliniques élargies (SpO2, températures) |
| 015 | `015_profile_identity.sql` | Neuf colonnes d'identité, inutilisées depuis le 2026-08-02 (réduction de périmètre voulue) |
| 016 | `016_clothing_sizes_estimated.sql` | Source `estimated` des tailles |
| 017 | `017_index_et_cascades.sql` | Sept index en double retirés ; `ON DELETE CASCADE` sur `alert_events` et `medication_logs` |
| 018 | `018_tables_v2_mortes.sql` | `derived_metrics` et `metric_targets` retirées (jamais lues, aucune mesure) |
| 019 | `019_sources.sql` | `sources`, `source_auth`, `source_credentials`, `source_state`, `metric_source_priority` ; trois vues de compatibilité |
| 020 | `020_cle_source.sql` | Clé étrangère `source` et `external_id` sur les huit tables de mesures |
| 021 | `021_journal_synchro_source.sql` | `sync_log.source` et son index |
| 022 | `022_appareil_d_origine.sql` | Colonne `device` |
| 023 | `023_traces_et_correspondances_csv.sql` | Source `trace`, table `csv_mappings` |

Les tables de la V2.0 de `bootstrap_schema` (`sync_log`, `journal_entries`, `insights`, `dashboard_layout`) sont créées par la migration 006, pas par du code.

Pourquoi les semis sont du Rust : au moment où une migration versionnée tourne, `measurement_types` peut déjà être la vue de rétrocompatibilité, et SQLite refuse un `INSERT` dans une vue. `seed_body_catalog` cible donc `measurement_types_legacy`, après le renommage, de façon idempotente (`INSERT OR IGNORE` puis mise à jour des métadonnées).

Preuves de chaque migration à reconstruction : jouée sur une copie de la base de développement, `integrity_check` et `foreign_key_check` propres avant et après, comptes de lignes égaux table par table (journaux de `logs/` des runs R27, R28, R33).

## 6. Tables résiduelles et réservées

Une base ancienne peut porter des tables qu'aucune migration actuelle ne crée et que le code ne lit pas : `vitals_v2`, `heart_records_v2`, `sync_state_v2` (restes d'un renommage refusé en 002 à 004), `vitals_repair`, `heart_records_repair`, `sync_state_repair` (script de réparation manuel retiré), `_vitals_new`, `_vitals_tmp`, `_sync_state_tmp`. Elles sont laissées : le dépôt ne sait pas ce que contient la base réelle et un `DROP` non prouvé est exclu. Pour le savoir, une requête en lecture seule suffit (`SELECT name FROM sqlite_master WHERE type = 'table' AND (name LIKE '%v2' OR name LIKE '%repair' OR name LIKE '%tmp' OR name LIKE '%new')`, puis un `COUNT(*)` par table).

`mood_logs`, `nutrition_logs`, `dashboard_layout` et la colonne `settings.theme` restent dans le schéma sans lecture ni écriture. Les retirer demande une reconstruction de table, un risque sans bénéfice : seule une migration décidée le fera, comme pour les colonnes de la 015.

## 7. Maintenance et formats

- **Statistiques** : `PRAGMA optimize=0x10002` à l'ouverture, `PRAGMA optimize` après chaque sauvegarde quotidienne écrite ; un échec est un `warn`.
- **Journaux** : `sync_log` borné (§ 3) ; `error_log` et `audit_log` purgés au démarrage et après la sauvegarde du jour (`services::retention`).
- **Détecteurs** : `measured_at >= date('now', '-30 days')` et non un `SUBSTR` sur la colonne, qui interdit l'index ; `tests/integration/sqlite_maintenance.rs` lit `EXPLAIN QUERY PLAN` et refuse un `SCAN`.
- **Dates** : `AAAA-MM-JJ` pour une date, ISO 8601 UTC avec `Z` pour un horodatage. `date` sert aux données quotidiennes (sommeil, activité, entraînements, journal), `measured_at` aux mesures ponctuelles, `timestamp` à `heart_records`. Pour comparer, on normalise le **paramètre** (`SUBSTR(?, 1, 10)`), jamais la colonne. `sync_log` garde ses formats (RFC 3339 pour `started_at`) et ne se compare qu'à lui-même.
- **Siestes** : `sleep_records` garde `UNIQUE(date, source)`. `sync_sleep` journalise un `warn` si une réponse Withings porte plusieurs séries pour une date (une seule est gardée).
- **Écritures longues** : `db/gate.rs` ramène le délai d'occupation de 5 s à 250 ms pour toute écriture pendant un import et refuse les synchros.

## 8. Sauvegardes

Toute copie passe par `services::backup::copy_database_safely` : `VACUUM INTO ?` vers `<dest>.partial` (création exclusive), réouverture en lecture seule, `PRAGMA integrity_check` égal à `ok` et au moins une table, puis renommage. Un `std::fs::copy` direct d'une base en WAL peut donner un fichier vide.

| Dossier | Quand | Conservées |
|---|---|---|
| `backups/auto/` | avant une migration due, nom `synk_{ISO}_pre-v{n}.db` ; l'échec arrête le démarrage | 3 |
| `backups/daily/` | quotidienne, si la dernière copie a plus de 20 h, vérifiée toutes les heures | 14 |
| `backups/` | manuelle (`create_backup`) | 5 |

La restauration (`restore_latest_backup`) prend, parmi les trois dossiers, la plus récente qui passe `verify_backup`, la copie en `synk.db.restoring`, déplace la base courante et ses `-wal` et `-shm` en `synk.db.broken-{ISO}`, puis renomme la copie ; si le renommage échoue, la base courante revient. Un échec de sauvegarde quotidienne va dans `error_log` (code `BACKUP_FAILED`), dans l'événement `backup-failed` et dans un état relisible (`get_backup_failure`).

## 9. Motifs d'écriture

Les données synchronisées s'écrivent par `INSERT ... ON CONFLICT ... DO UPDATE` sur leur clé d'unicité :

| Table | Clé de conflit |
|---|---|
| `body_measures_legacy`, `blood_pressure` | `withings_grpid` (Withings) ; `(source, external_id)` (autres sources) |
| `vitals_legacy` | `(withings_grpid, vital_type)` ; `(source, external_id)` |
| `sleep_records`, `activity` | `(date, source)` |
| `workouts` | `(date, withings_id)` ; `(source, external_id)` |
| `heart_records` | `(timestamp, source)` |
| `withings_devices` | `device_id` |

`settings` est un singleton (`CHECK (id = 1)`). Une requête d'import `INSERT OR IGNORE` doit nommer la colonne qui porte l'unicité, sinon rien n'est dédoublonné (deux `NULL` ne sont jamais égaux).
