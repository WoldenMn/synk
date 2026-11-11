import { invoke } from "@tauri-apps/api/core";

/**
 * Retourne le chemin absolu du dossier `app_data_dir` Tauri (ex:
 * `%APPDATA%/com.synk.health` sur Windows).
 */
export function getAppDataDir(): Promise<string> {
  return invoke<string>("get_app_data_dir");
}
