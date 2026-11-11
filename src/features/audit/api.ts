import { invoke } from "@tauri-apps/api/core";
import type { AuditMetrics, AuditPurgeConfig } from "./types";

export function getAuditMetrics(): Promise<AuditMetrics> {
  return invoke<AuditMetrics>("get_audit_metrics");
}

export function purgeAuditLog(olderThanDays: number): Promise<number> {
  return invoke<number>("purge_audit_log", { olderThanDays });
}

export function getAuditPurgeConfig(): Promise<AuditPurgeConfig> {
  return invoke<AuditPurgeConfig>("get_audit_purge_config");
}

export function setAuditPurgeConfig(config: AuditPurgeConfig): Promise<void> {
  return invoke<void>("set_audit_purge_config", { config });
}
