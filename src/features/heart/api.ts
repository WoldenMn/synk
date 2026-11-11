import { invoke } from "@tauri-apps/api/core";
import type { HeartRecord } from "./types";

export function getHeartRecords(from: string, to: string): Promise<HeartRecord[]> {
  return invoke<HeartRecord[]>("get_heart_records", { from, to });
}

export function getLatestHeartRecord(): Promise<HeartRecord | null> {
  return invoke<HeartRecord | null>("get_latest_heart_record");
}
