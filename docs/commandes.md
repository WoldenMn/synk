# Commandes IPC

Fichier généré par `npm run doc:commandes` depuis `src-tauri/src/lib.rs` et les signatures Rust : ne pas l'éditer à la main. Le gardien `src/__tests__/doc-drift.test.ts` le compare au code à chaque `npm run verify`.

111 commandes.

| Commande | Arguments (côté front, en camelCase ; `?` = facultatif) | Fichier |
|---|---|---|
| `add_alert` | data | `src-tauri/src/commands/alerts.rs` |
| `add_lab_result` | data | `src-tauri/src/commands/labs.rs` |
| `add_manual_activity` | data | `src-tauri/src/commands/activity.rs` |
| `add_manual_blood_pressure` | data | `src-tauri/src/commands/blood_pressure.rs` |
| `add_manual_body_measure` | data | `src-tauri/src/commands/body_measures.rs` |
| `add_manual_measurement` | data | `src-tauri/src/commands/manual_measurements/measurements.rs` |
| `add_manual_sleep` | data | `src-tauri/src/commands/sleep.rs` |
| `add_manual_workout` | data | `src-tauri/src/commands/workouts.rs` |
| `add_measurement_session` | measuredAt, entries | `src-tauri/src/commands/body_profile.rs` |
| `add_measurement_type` | data | `src-tauri/src/commands/manual_measurements/types.rs` |
| `add_medication` | data | `src-tauri/src/commands/medications.rs` |
| `cancel_import` | aucun | `src-tauri/src/commands/import.rs` |
| `clear_sync_log` | aucun | `src-tauri/src/commands/sync_log.rs` |
| `create_backup` | aucun | `src-tauri/src/commands/export/backup.rs` |
| `delete_activity` | id | `src-tauri/src/commands/activity.rs` |
| `delete_alert` | id | `src-tauri/src/commands/alerts.rs` |
| `delete_blood_pressure` | id | `src-tauri/src/commands/blood_pressure.rs` |
| `delete_body_measure` | id | `src-tauri/src/commands/body_measures.rs` |
| `delete_clothing_size` | category | `src-tauri/src/commands/clothing_sizes.rs` |
| `delete_csv_mapping` | id | `src-tauri/src/commands/import.rs` |
| `delete_goal` | metricKey | `src-tauri/src/commands/goals.rs` |
| `delete_journal_entry` | id | `src-tauri/src/commands/journal.rs` |
| `delete_lab_result` | id | `src-tauri/src/commands/labs.rs` |
| `delete_manual_measurement` | id | `src-tauri/src/commands/manual_measurements/measurements.rs` |
| `delete_measurement_type` | id | `src-tauri/src/commands/manual_measurements/types.rs` |
| `delete_medication` | id | `src-tauri/src/commands/medications.rs` |
| `delete_sleep_record` | id | `src-tauri/src/commands/sleep.rs` |
| `delete_workout` | id | `src-tauri/src/commands/workouts.rs` |
| `evaluate_alerts` | aucun | `src-tauri/src/commands/alerts.rs` |
| `export_body_sheet_pdf` | aucun | `src-tauri/src/commands/export/pdf/sheet.rs` |
| `export_csv` | from, to, categories | `src-tauri/src/commands/export/csv/mod.rs` |
| `export_json` | aucun | `src-tauri/src/commands/export/json/export.rs` |
| `export_pdf` | from, to, categories | `src-tauri/src/commands/export/pdf/mod.rs` |
| `export_profile_card_pdf` | aucun | `src-tauri/src/commands/export/pdf/profile/mod.rs` |
| `generate_insights` | aucun | `src-tauri/src/commands/insights/mod.rs` |
| `get_activity` | from, to | `src-tauri/src/commands/activity.rs` |
| `get_alert_events` | aucun | `src-tauri/src/commands/alerts.rs` |
| `get_alerts` | aucun | `src-tauri/src/commands/alerts.rs` |
| `get_app_data_dir` | aucun | `src-tauri/src/commands/settings.rs` |
| `get_audit_metrics` | aucun | `src-tauri/src/commands/audit.rs` |
| `get_audit_purge_config` | aucun | `src-tauri/src/commands/audit.rs` |
| `get_backup_failure` | aucun | `src-tauri/src/boot/periodic_backup.rs` |
| `get_blood_pressure` | from, to | `src-tauri/src/commands/blood_pressure.rs` |
| `get_body_measures` | from, to | `src-tauri/src/commands/body_measures.rs` |
| `get_body_sheet` | aucun | `src-tauri/src/commands/body_profile.rs` |
| `get_boot_state` | aucun | `src-tauri/src/boot/runner.rs` |
| `get_clothing_inputs` | aucun | `src-tauri/src/commands/clothing_sizes.rs` |
| `get_clothing_sizes` | aucun | `src-tauri/src/commands/clothing_sizes.rs` |
| `get_dashboard_cards` | from, to | `src-tauri/src/commands/dashboard/cards.rs` |
| `get_db_metrics` | aucun | `src-tauri/src/commands/dashboard/metrics.rs` |
| `get_error_log` | limit, codeFilter? | `src-tauri/src/commands/error_log.rs` |
| `get_goals` | aucun | `src-tauri/src/commands/goals.rs` |
| `get_heart_records` | from, to | `src-tauri/src/commands/heart.rs` |
| `get_insights` | limit? | `src-tauri/src/commands/insights/mod.rs` |
| `get_invalidated_secrets` | aucun | `src-tauri/src/commands/sources.rs` |
| `get_journal_entries` | from?, to? | `src-tauri/src/commands/journal.rs` |
| `get_lab_results` | from, to | `src-tauri/src/commands/labs.rs` |
| `get_last_successful_sync` | aucun | `src-tauri/src/commands/sync_log.rs` |
| `get_latest_activity` | aucun | `src-tauri/src/commands/activity.rs` |
| `get_latest_blood_pressure` | aucun | `src-tauri/src/commands/blood_pressure.rs` |
| `get_latest_body_measure` | aucun | `src-tauri/src/commands/body_measures.rs` |
| `get_latest_heart_record` | aucun | `src-tauri/src/commands/heart.rs` |
| `get_latest_sleep` | aucun | `src-tauri/src/commands/sleep.rs` |
| `get_latest_workout` | aucun | `src-tauri/src/commands/workouts.rs` |
| `get_manual_measurements` | typeId, from, to | `src-tauri/src/commands/manual_measurements/measurements.rs` |
| `get_measurement_overview` | from, to | `src-tauri/src/commands/manual_measurements/stats.rs` |
| `get_measurement_stats` | typeId, from, to | `src-tauri/src/commands/manual_measurements/stats.rs` |
| `get_measurement_types` | aucun | `src-tauri/src/commands/manual_measurements/types.rs` |
| `get_medication_logs` | medicationId, from, to | `src-tauri/src/commands/medications.rs` |
| `get_medications` | aucun | `src-tauri/src/commands/medications.rs` |
| `get_metric_definitions` | category? | `src-tauri/src/commands/metrics.rs` |
| `get_metric_latest` | slug | `src-tauri/src/commands/metrics.rs` |
| `get_metric_priorities` | aucun | `src-tauri/src/commands/metrics.rs` |
| `get_metric_secondary_points` | slug, from, to | `src-tauri/src/commands/metrics.rs` |
| `get_metric_series` | slug, from, to | `src-tauri/src/commands/metrics.rs` |
| `get_profile_card` | aucun | `src-tauri/src/commands/profile_card/mod.rs` |
| `get_sleep_records` | from, to | `src-tauri/src/commands/sleep.rs` |
| `get_sync_log` | limit? | `src-tauri/src/commands/sync_log.rs` |
| `get_user_profile` | aucun | `src-tauri/src/commands/profile.rs` |
| `get_withings_devices` | aucun | `src-tauri/src/commands/devices.rs` |
| `get_workouts` | from, to | `src-tauri/src/commands/workouts.rs` |
| `import_csv_file` | path, config | `src-tauri/src/commands/import.rs` |
| `import_json` | path | `src-tauri/src/commands/export/json/import.rs` |
| `import_source_file` | source, path | `src-tauri/src/commands/import.rs` |
| `inspect_csv_file` | path | `src-tauri/src/commands/import.rs` |
| `inspect_import_file` | path | `src-tauri/src/commands/export/json/import.rs` |
| `list_backups` | aucun | `src-tauri/src/commands/export/backup_files.rs` |
| `list_sources` | aucun | `src-tauri/src/commands/sources.rs` |
| `log_error` | source, code, message, context? | `src-tauri/src/commands/error_log.rs` |
| `log_medication_taken` | data | `src-tauri/src/commands/medications.rs` |
| `mark_alerts_seen` | aucun | `src-tauri/src/commands/alerts.rs` |
| `mark_insights_seen` | aucun | `src-tauri/src/commands/insights/mod.rs` |
| `purge_audit_log` | olderThanDays | `src-tauri/src/commands/audit.rs` |
| `purge_error_log` | aucun | `src-tauri/src/commands/error_log.rs` |
| `restore_latest_backup` | aucun | `src-tauri/src/commands/export/restore.rs` |
| `retry_boot` | aucun | `src-tauri/src/boot/runner.rs` |
| `save_csv_mapping` | name, signature, config | `src-tauri/src/commands/import.rs` |
| `save_estimated_sizes` | sizes | `src-tauri/src/commands/clothing_sizes.rs` |
| `set_audit_purge_config` | config | `src-tauri/src/commands/audit.rs` |
| `set_clothing_size` | category, value, source | `src-tauri/src/commands/clothing_sizes.rs` |
| `set_measurement_goal` | typeId, targetValue?, targetDirection? | `src-tauri/src/commands/manual_measurements/goals.rs` |
| `set_metric_priority` | slug, sources | `src-tauri/src/commands/metrics.rs` |
| `source_disconnect` | source | `src-tauri/src/commands/sources.rs` |
| `source_set_credentials` | source, clientId, clientSecret | `src-tauri/src/commands/sources.rs` |
| `source_start_auth` | source | `src-tauri/src/commands/sources.rs` |
| `sync_all_sources` | aucun | `src-tauri/src/commands/sources.rs` |
| `sync_source` | source | `src-tauri/src/commands/sources.rs` |
| `update_medication` | data | `src-tauri/src/commands/medications.rs` |
| `update_user_profile` | age?, sex?, heightCm? | `src-tauri/src/commands/profile.rs` |
| `upsert_goal` | data | `src-tauri/src/commands/goals.rs` |
| `upsert_journal_entry` | data | `src-tauri/src/commands/journal.rs` |
