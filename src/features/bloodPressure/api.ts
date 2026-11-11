import { invoke } from "@tauri-apps/api/core";
import type { BloodPressure, ManualBloodPressureInput } from "./types";

export function getBloodPressure(from: string, to: string): Promise<BloodPressure[]> {
  return invoke<BloodPressure[]>("get_blood_pressure", { from, to });
}

export function getLatestBloodPressure(): Promise<BloodPressure | null> {
  return invoke<BloodPressure | null>("get_latest_blood_pressure");
}

export function addManualBloodPressure(data: ManualBloodPressureInput): Promise<number> {
  return invoke<number>("add_manual_blood_pressure", { data });
}

export function deleteBloodPressure(id: number): Promise<void> {
  return invoke("delete_blood_pressure", { id });
}
