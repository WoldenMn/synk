export interface Goal {
  id: number;
  metric_key: string;
  target_value: number;
  direction: string;
}

export interface UpsertGoalInput {
  metric_key: string;
  target_value: number;
  direction: string;
}
