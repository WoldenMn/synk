import { invoke } from "@tauri-apps/api/core";
import type {
  ImportReport,
  InvalidatedSecrets,
  MetricPriority,
  SourceInfo,
  SourceOutcome,
} from "./types";

export function listSources(): Promise<SourceInfo[]> {
  return invoke<SourceInfo[]>("list_sources");
}

export function sourceStartAuth(source: string): Promise<string> {
  return invoke<string>("source_start_auth", { source });
}

export interface DisconnectReport {
  revoked: boolean;
  warning: string | null;
}

export function sourceDisconnect(source: string): Promise<DisconnectReport> {
  return invoke<DisconnectReport>("source_disconnect", { source });
}

export function sourceSetCredentials(
  source: string,
  clientId: string,
  clientSecret: string
): Promise<void> {
  return invoke<void>("source_set_credentials", { source, clientId, clientSecret });
}

export function syncAllSources(): Promise<SourceOutcome[]> {
  return invoke<SourceOutcome[]>("sync_all_sources");
}

export function syncSource(source: string): Promise<SourceOutcome[]> {
  return invoke<SourceOutcome[]>("sync_source", { source });
}

export function getInvalidatedSecrets(): Promise<InvalidatedSecrets> {
  return invoke<InvalidatedSecrets>("get_invalidated_secrets");
}

export function getMetricPriorities(): Promise<MetricPriority[]> {
  return invoke<MetricPriority[]>("get_metric_priorities");
}

export function setMetricPriority(slug: string, sources: string[]): Promise<void> {
  return invoke<void>("set_metric_priority", { slug, sources });
}

export function importSourceFile(source: string, path: string): Promise<ImportReport> {
  return invoke<ImportReport>("import_source_file", { source, path });
}

export function cancelImport(): Promise<boolean> {
  return invoke<boolean>("cancel_import");
}
