import { invoke } from "@tauri-apps/api/core";
import type { RustError } from "@/lib/errors";

export interface BackupInfo {
  path: string;
  kind: "auto" | "quotidienne" | "manuelle";
  size_bytes: number;
  modified: string;
}

export function getBackupFailure(): Promise<RustError | null> {
  return invoke<RustError | null>("get_backup_failure");
}

export function listBackups(): Promise<BackupInfo[]> {
  return invoke<BackupInfo[]>("list_backups");
}
