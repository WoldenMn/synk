export interface MeasurementType {
  id: number;
  slug: string;
  label: string;
  unit: string;
  category: string;
  sort_order: number;
  is_builtin: number;
  target_value: number | null;
  target_direction: string | null;
}

export interface MeasurementStats {
  latest_value: number | null;
  latest_date: string | null;
  min_value: number | null;
  max_value: number | null;
  avg_value: number | null;
  count: number;
  prev_value: number | null;
  percent_change: number | null;
}

export interface AddMeasurementTypeInput {
  slug: string;
  label: string;
  unit: string;
}

export interface ManualMeasurement {
  id: number;
  measurement_type_id: number;
  measured_at: string;
  value: number;
  notes: string | null;
}

export interface AddManualMeasurementInput {
  measurement_type_id: number;
  measured_at: string;
  value: number;
  notes: string | null;
}

export interface MeasurementOverview {
  type_id: number;
  stats: MeasurementStats;
  values: number[];
}
