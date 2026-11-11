import { invoke } from "@tauri-apps/api/core";
import type {
  MeasurementType,
  AddMeasurementTypeInput,
  ManualMeasurement,
  AddManualMeasurementInput,
  MeasurementStats,
  MeasurementOverview,
} from "./types";

export function getMeasurementTypes(): Promise<MeasurementType[]> {
  return invoke<MeasurementType[]>("get_measurement_types");
}

export function addMeasurementType(data: AddMeasurementTypeInput): Promise<number> {
  return invoke<number>("add_measurement_type", { data });
}

export function deleteMeasurementType(id: number): Promise<void> {
  return invoke("delete_measurement_type", { id });
}

export function getManualMeasurements(
  typeId: number,
  from: string,
  to: string
): Promise<ManualMeasurement[]> {
  return invoke<ManualMeasurement[]>("get_manual_measurements", { typeId, from, to });
}

export function addManualMeasurement(data: AddManualMeasurementInput): Promise<number> {
  return invoke<number>("add_manual_measurement", { data });
}

export function deleteManualMeasurement(id: number): Promise<void> {
  return invoke("delete_manual_measurement", { id });
}

export function getMeasurementStats(
  typeId: number,
  from: string,
  to: string
): Promise<MeasurementStats> {
  return invoke<MeasurementStats>("get_measurement_stats", { typeId, from, to });
}

export function getMeasurementOverview(from: string, to: string): Promise<MeasurementOverview[]> {
  return invoke<MeasurementOverview[]>("get_measurement_overview", { from, to });
}

export function setMeasurementGoal(
  typeId: number,
  targetValue: number | null,
  targetDirection: string | null
): Promise<void> {
  return invoke("set_measurement_goal", { typeId, targetValue, targetDirection });
}
