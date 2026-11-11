#![deny(clippy::string_slice, clippy::let_underscore_must_use)]
#![cfg_attr(not(test), warn(clippy::too_many_lines, clippy::excessive_nesting))]
pub mod body_catalog;
pub mod boot;
pub mod commands;
pub mod crypto;
pub mod db;
pub mod error;
pub mod logging;
pub mod metric_legacy;
pub mod metric_model;
pub mod metric_repository;
pub mod services;
pub mod sources;
pub mod utils;
pub mod validation;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(clippy::too_many_lines, reason = "liste des commandes enregistrees")]
pub fn run() {
    let (log_level, log_warning) =
        logging::level_from(std::env::var(logging::ENV_VAR).ok().as_deref());
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.unminimize().and_then(|()| window.set_focus()) {
                    log::warn!(
                        "[single-instance] fenetre existante non ramenee au premier plan : {e}"
                    );
                }
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log_level)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(
                    logging::FILES_KEPT,
                ))
                .max_file_size(logging::FILE_MAX_BYTES)
                .build(),
        )
        .setup(move |app| {
            if let Some(warning) = &log_warning {
                log::warn!("[boot] {warning}");
            }
            boot::container::log_launch();
            if cfg!(debug_assertions) {
                if let Some(window) = app.get_webview_window("main") {
                    if let Err(e) = window.set_title("Synk (dev)") {
                        log::warn!("[dev] titre de fenetre non pose : {e}");
                    }
                }
            }
            log::info!("[boot] setup() entered, spawning boot::run");
            let snapshot = Arc::new(RwLock::new(boot::runner::BootSnapshot::Loading));
            app.manage(boot::runner::AppBootState(snapshot.clone()));
            app.manage(boot::BackupFailure::default());
            app.manage(commands::import::ImportControl::default());
            boot::runner::spawn_boot(app.handle().clone(), snapshot);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            boot::runner::get_boot_state,
            boot::runner::retry_boot,
            boot::get_backup_failure,
            commands::settings::get_app_data_dir,
            commands::body_measures::get_body_measures,
            commands::body_measures::get_latest_body_measure,
            commands::body_measures::add_manual_body_measure,
            commands::body_measures::delete_body_measure,
            commands::blood_pressure::get_blood_pressure,
            commands::blood_pressure::get_latest_blood_pressure,
            commands::blood_pressure::add_manual_blood_pressure,
            commands::blood_pressure::delete_blood_pressure,
            commands::sleep::get_sleep_records,
            commands::sleep::get_latest_sleep,
            commands::sleep::add_manual_sleep,
            commands::sleep::delete_sleep_record,
            commands::activity::get_activity,
            commands::activity::get_latest_activity,
            commands::activity::add_manual_activity,
            commands::activity::delete_activity,
            commands::manual_measurements::get_measurement_types,
            commands::manual_measurements::add_measurement_type,
            commands::manual_measurements::delete_measurement_type,
            commands::manual_measurements::get_manual_measurements,
            commands::manual_measurements::add_manual_measurement,
            commands::manual_measurements::delete_manual_measurement,
            commands::manual_measurements::get_measurement_stats,
            commands::manual_measurements::get_measurement_overview,
            commands::manual_measurements::set_measurement_goal,
            commands::body_profile::add_measurement_session,
            commands::body_profile::get_body_sheet,
            commands::export::export_body_sheet_pdf,
            commands::goals::get_goals,
            commands::goals::upsert_goal,
            commands::goals::delete_goal,
            commands::dashboard::get_dashboard_cards,
            commands::export::export_csv,
            commands::export::export_pdf,
            commands::workouts::get_workouts,
            commands::workouts::get_latest_workout,
            commands::workouts::add_manual_workout,
            commands::workouts::delete_workout,
            commands::heart::get_heart_records,
            commands::heart::get_latest_heart_record,
            commands::devices::get_withings_devices,
            commands::profile::get_user_profile,
            commands::profile::update_user_profile,
            commands::profile_card::get_profile_card,
            commands::export::export_profile_card_pdf,
            commands::alerts::get_alerts,
            commands::alerts::add_alert,
            commands::alerts::delete_alert,
            commands::alerts::get_alert_events,
            commands::alerts::mark_alerts_seen,
            commands::alerts::evaluate_alerts,
            commands::export::export_json,
            commands::export::import_json,
            commands::export::inspect_import_file,
            commands::export::create_backup,
            commands::export::list_backups,
            commands::export::restore_latest_backup,
            commands::dashboard::get_db_metrics,
            commands::audit::get_audit_metrics,
            commands::audit::purge_audit_log,
            commands::audit::get_audit_purge_config,
            commands::audit::set_audit_purge_config,
            commands::medications::get_medications,
            commands::medications::add_medication,
            commands::medications::update_medication,
            commands::medications::delete_medication,
            commands::medications::log_medication_taken,
            commands::medications::get_medication_logs,
            commands::labs::get_lab_results,
            commands::labs::add_lab_result,
            commands::labs::delete_lab_result,
            commands::sync_log::get_sync_log,
            commands::sync_log::get_last_successful_sync,
            commands::sync_log::clear_sync_log,
            commands::insights::get_insights,
            commands::insights::mark_insights_seen,
            commands::insights::generate_insights,
            commands::journal::get_journal_entries,
            commands::journal::upsert_journal_entry,
            commands::journal::delete_journal_entry,
            commands::metrics::get_metric_definitions,
            commands::metrics::get_metric_series,
            commands::metrics::get_metric_latest,
            commands::metrics::get_metric_priorities,
            commands::metrics::get_metric_secondary_points,
            commands::metrics::set_metric_priority,
            commands::sources::list_sources,
            commands::sources::source_start_auth,
            commands::sources::source_disconnect,
            commands::sources::source_set_credentials,
            commands::sources::sync_all_sources,
            commands::sources::get_invalidated_secrets,
            commands::sources::sync_source,
            commands::import::import_source_file,
            commands::import::cancel_import,
            commands::import::import_csv_file,
            commands::import::inspect_csv_file,
            commands::import::save_csv_mapping,
            commands::import::delete_csv_mapping,
            commands::clothing_sizes::get_clothing_inputs,
            commands::clothing_sizes::get_clothing_sizes,
            commands::clothing_sizes::set_clothing_size,
            commands::clothing_sizes::delete_clothing_size,
            commands::clothing_sizes::save_estimated_sizes,
            commands::error_log::log_error,
            commands::error_log::get_error_log,
            commands::error_log::purge_error_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
