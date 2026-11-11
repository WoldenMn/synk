import { invoke } from "@tauri-apps/api/core";

export interface ExportOutcome {
  path: string | null;
  rows: number;
}

export interface TableImport {
  key: string;
  inserted: number;
  ignored: number;
  updated: number;
  orphans: number;
}

export interface ImportPreview {
  key: string;
  rows: number;
}

export function exportCsv(from: string, to: string, categories: string[]): Promise<ExportOutcome> {
  return invoke<ExportOutcome>("export_csv", { from, to, categories });
}

export function exportPdf(from: string, to: string, categories: string[]): Promise<ExportOutcome> {
  return invoke<ExportOutcome>("export_pdf", { from, to, categories });
}

export function inspectImportFile(path: string): Promise<ImportPreview[]> {
  return invoke<ImportPreview[]>("inspect_import_file", { path });
}

export function exportJson(): Promise<string> {
  return invoke<string>("export_json");
}

export function importJson(path: string): Promise<{ tables: TableImport[] }> {
  return invoke<{ tables: TableImport[] }>("import_json", { path });
}

export function createBackup(): Promise<string> {
  return invoke<string>("create_backup");
}
