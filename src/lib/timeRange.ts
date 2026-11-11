export type TimeRangeValue = "7d" | "30d" | "90d" | "1y" | "all";

export interface TimeRange {
  label: string;
  value: TimeRangeValue;
  days: number;
}
