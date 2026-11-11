import { invoke } from "@tauri-apps/api/core";
import type { SleepRecord, ManualSleepInput } from "./types";

export function getSleepRecords(from: string, to: string): Promise<SleepRecord[]> {
  return invoke<SleepRecord[]>("get_sleep_records", { from, to });
}

export function getLatestSleep(): Promise<SleepRecord | null> {
  return invoke<SleepRecord | null>("get_latest_sleep");
}

export function addManualSleep(data: ManualSleepInput): Promise<number> {
  return invoke<number>("add_manual_sleep", { data });
}

export function deleteSleepRecord(id: number): Promise<void> {
  return invoke("delete_sleep_record", { id });
}
