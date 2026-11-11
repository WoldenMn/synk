import { invoke } from "@tauri-apps/api/core";

export interface SyncLogEntry {
  id: number;
  source: string;
  started_at: string;
  finished_at: string | null;
  data_type: string;
  duration_ms: number | null;
  records_added: number | null;
  status: string;
  error: string | null;
}

export function getSyncLog(limit?: number): Promise<SyncLogEntry[]> {
  return invoke<SyncLogEntry[]>("get_sync_log", { limit: limit ?? 100 });
}

export function clearSyncLog(): Promise<void> {
  return invoke("clear_sync_log");
}

export function getLastSuccessfulSync(): Promise<string | null> {
  return invoke<string | null>("get_last_successful_sync");
}
