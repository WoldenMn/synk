import { invoke } from "@tauri-apps/api/core";
import type { CsvConfig, CsvInspection, ImportReport } from "../types";

export function inspectCsvFile(path: string): Promise<CsvInspection> {
  return invoke<CsvInspection>("inspect_csv_file", { path });
}

export function importCsvFile(path: string, config: CsvConfig): Promise<ImportReport> {
  return invoke<ImportReport>("import_csv_file", { path, config });
}

export function saveCsvMapping(
  name: string,
  signature: string,
  config: CsvConfig
): Promise<number> {
  return invoke<number>("save_csv_mapping", { name, signature, config });
}

export function deleteCsvMapping(id: number): Promise<boolean> {
  return invoke<boolean>("delete_csv_mapping", { id });
}
