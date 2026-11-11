export type MetricCategory =
  | "body_composition"
  | "circumference"
  | "body_scan"
  | "derived"
  | "custom";

export interface MetricDefinition {
  id: number;
  slug: string;
  label: string;
  unit: string;
  category: MetricCategory;
  withings_type: number | null;
  is_builtin: number;
  sort_order: number;
  decimals: number;
  min_value: number | null;
  max_value: number | null;
}

export interface MetricPoint {
  measured_at: string;
  value: number;
  source: string;
}
