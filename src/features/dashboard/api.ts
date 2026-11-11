import { invoke } from "@tauri-apps/api/core";
import type { DashboardCards, DbMetrics } from "@/features/dashboard/types";

export function getDbMetrics(): Promise<DbMetrics> {
  return invoke<DbMetrics>("get_db_metrics");
}

export function getDashboardCards(from: string, to: string): Promise<DashboardCards> {
  return invoke<DashboardCards>("get_dashboard_cards", { from, to });
}
