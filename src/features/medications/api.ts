import { invoke } from "@tauri-apps/api/core";
import type {
  Medication,
  AddMedicationInput,
  UpdateMedicationInput,
  MedicationLog,
  LogMedicationInput,
} from "./types";

export function getMedications(): Promise<Medication[]> {
  return invoke<Medication[]>("get_medications");
}

export function addMedication(data: AddMedicationInput): Promise<number> {
  return invoke<number>("add_medication", { data });
}

export function updateMedication(data: UpdateMedicationInput): Promise<void> {
  return invoke("update_medication", { data });
}

export function deleteMedication(id: number): Promise<void> {
  return invoke("delete_medication", { id });
}

export function logMedicationTaken(data: LogMedicationInput): Promise<number> {
  return invoke<number>("log_medication_taken", { data });
}

export function getMedicationLogs(
  medicationId: number,
  from: string,
  to: string
): Promise<MedicationLog[]> {
  return invoke<MedicationLog[]>("get_medication_logs", { medicationId, from, to });
}
