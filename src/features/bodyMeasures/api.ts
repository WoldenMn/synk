import { invoke } from "@tauri-apps/api/core";
import type { BodyMeasure, ManualBodyMeasureInput } from "./types";

export function getBodyMeasures(from: string, to: string): Promise<BodyMeasure[]> {
  return invoke<BodyMeasure[]>("get_body_measures", { from, to });
}

export function getLatestBodyMeasure(): Promise<BodyMeasure | null> {
  return invoke<BodyMeasure | null>("get_latest_body_measure");
}

export function addManualBodyMeasure(data: ManualBodyMeasureInput): Promise<number> {
  return invoke<number>("add_manual_body_measure", { data });
}

export function deleteBodyMeasure(id: number): Promise<void> {
  return invoke("delete_body_measure", { id });
}
