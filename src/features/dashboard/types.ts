import type { BodyMeasure } from "@/features/bodyMeasures/types";
import type { BloodPressure } from "@/features/bloodPressure/types";
import type { SleepRecord } from "@/features/sleep/types";
import type { Activity } from "@/features/activity/types";
import type { Workout } from "@/features/workouts/types";
import type { HeartRecord } from "@/features/heart/types";

export interface TrendData {
  current_value: number | null;
  previous_value: number | null;
  percent_change: number | null;
  direction: string;
}

export interface DashboardTrends {
  weight: TrendData;
  systolic: TrendData;
  sleep_score: TrendData;
  steps: TrendData;
}

export interface DashboardCards {
  body: BodyMeasure | null;
  blood_pressure: BloodPressure | null;
  sleep: SleepRecord | null;
  activity: Activity | null;
  workout: Workout | null;
  heart: HeartRecord | null;
  trends: DashboardTrends;
}

export interface DbMetrics {
  db_size_bytes: number;
  audit_log_count: number;
  body_measures_count: number;
  blood_pressure_count: number;
  sleep_count: number;
  activity_count: number;
  workouts_count: number;
}
