import { invoke } from "@tauri-apps/api/core";
import type { Goal, UpsertGoalInput } from "./types";

export function getGoals(): Promise<Goal[]> {
  return invoke<Goal[]>("get_goals");
}

export function upsertGoal(data: UpsertGoalInput): Promise<void> {
  return invoke("upsert_goal", { data });
}

export function deleteGoal(metricKey: string): Promise<void> {
  return invoke("delete_goal", { metricKey });
}
