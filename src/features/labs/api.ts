import { invoke } from "@tauri-apps/api/core";
import type { LabResult, AddLabResultInput } from "./types";

export function getLabResults(from: string, to: string): Promise<LabResult[]> {
  return invoke<LabResult[]>("get_lab_results", { from, to });
}

export function addLabResult(data: AddLabResultInput): Promise<number> {
  return invoke<number>("add_lab_result", { data });
}

export function deleteLabResult(id: number): Promise<void> {
  return invoke("delete_lab_result", { id });
}
