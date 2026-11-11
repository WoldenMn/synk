import { invoke } from "@tauri-apps/api/core";
import type { BootSnapshot, RestoreReport } from "./types";

export function getBootState(): Promise<BootSnapshot> {
  return invoke<BootSnapshot>("get_boot_state");
}

export function retryBoot(): Promise<void> {
  return invoke("retry_boot");
}

export function getAppDataDir(): Promise<string> {
  return invoke<string>("get_app_data_dir");
}

export function restoreLatestBackup(): Promise<RestoreReport> {
  return invoke<RestoreReport>("restore_latest_backup");
}
