import { invoke } from "@tauri-apps/api/core";
import type { Activity, ManualActivityInput } from "./types";

export function getActivity(from: string, to: string): Promise<Activity[]> {
  return invoke<Activity[]>("get_activity", { from, to });
}

export function getLatestActivity(): Promise<Activity | null> {
  return invoke<Activity | null>("get_latest_activity");
}

export function addManualActivity(data: ManualActivityInput): Promise<number> {
  return invoke<number>("add_manual_activity", { data });
}

export function deleteActivity(id: number): Promise<void> {
  return invoke("delete_activity", { id });
}
