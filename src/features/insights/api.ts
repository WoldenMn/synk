import { invoke } from "@tauri-apps/api/core";

export interface Insight {
  id: number;
  kind: string;
  severity: "info" | "warning" | "danger";
  metric: string | null;
  title: string;
  message: string;
  value: number | null;
  delta: number | null;
  generated_at: string;
  seen: number;
}

export function getInsights(limit?: number): Promise<Insight[]> {
  return invoke<Insight[]>("get_insights", { limit: limit ?? 50 });
}

export function markInsightsSeen(): Promise<void> {
  return invoke("mark_insights_seen");
}

export interface InsightReport {
  generated: number;
  failed: string[];
}

export function generateInsights(): Promise<InsightReport> {
  return invoke<InsightReport>("generate_insights");
}
