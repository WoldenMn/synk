import { invoke } from "@tauri-apps/api/core";
import type { Workout, ManualWorkoutInput } from "./types";

export function getWorkouts(from: string, to: string): Promise<Workout[]> {
  return invoke<Workout[]>("get_workouts", { from, to });
}

export function getLatestWorkout(): Promise<Workout | null> {
  return invoke<Workout | null>("get_latest_workout");
}

export function addManualWorkout(data: ManualWorkoutInput): Promise<number> {
  return invoke<number>("add_manual_workout", { data });
}

export function deleteWorkout(id: number): Promise<void> {
  return invoke("delete_workout", { id });
}
