import { invoke } from "@tauri-apps/api/core";
import type { ErrorLogEntry } from "./types";

export function getErrorLog(
  limit = 200,
  codeFilter: string | null = null
): Promise<ErrorLogEntry[]> {
  return invoke<ErrorLogEntry[]>("get_error_log", { limit, codeFilter });
}

export function logErrorEntry(entry: {
  source: string;
  code: string;
  message: string;
  context: string | null;
}): Promise<void> {
  return invoke("log_error", entry);
}

export function purgeErrorLog(): Promise<void> {
  return invoke("purge_error_log");
}
