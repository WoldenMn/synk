import { invoke } from "@tauri-apps/api/core";
import type { SessionEntry, SheetRow } from "./types";

export function getBodySheet(): Promise<SheetRow[]> {
  return invoke<SheetRow[]>("get_body_sheet");
}

export function addMeasurementSession(
  measuredAt: string,
  entries: SessionEntry[]
): Promise<number> {
  return invoke<number>("add_measurement_session", { measuredAt, entries });
}

export function exportBodySheetPdf(): Promise<string> {
  return invoke<string>("export_body_sheet_pdf");
}
