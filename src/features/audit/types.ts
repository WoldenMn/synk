export interface AuditMetrics {
  total_count: number;
  insert_count: number;
  delete_count: number;
  oldest_at: string | null;
  newest_at: string | null;
}

export interface AuditPurgeConfig {
  retention_days: number;
  max_entries: number;
  enabled: boolean;
}
