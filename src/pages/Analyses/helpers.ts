export interface MetricOption {
  value: string;
  label: string;
  unit: string;
  source: "body" | "bp" | "sleep" | "activity";
  field: string;
}

const FALLBACK_METRIC: MetricOption = {
  value: "weight",
  label: "Poids",
  unit: "kg",
  source: "body",
  field: "weight",
};

export const METRIC_OPTIONS: MetricOption[] = [
  FALLBACK_METRIC,
  { value: "fat_ratio", label: "Masse grasse", unit: "%", source: "body", field: "fat_ratio" },
  {
    value: "muscle_mass",
    label: "Masse musculaire",
    unit: "kg",
    source: "body",
    field: "muscle_mass",
  },
  { value: "systolic", label: "Systolique", unit: "mmHg", source: "bp", field: "systolic" },
  { value: "diastolic", label: "Diastolique", unit: "mmHg", source: "bp", field: "diastolic" },
  {
    value: "heart_rate",
    label: "Fréquence cardiaque",
    unit: "bpm",
    source: "bp",
    field: "heart_rate",
  },
  {
    value: "sleep_score",
    label: "Score sommeil",
    unit: "pts",
    source: "sleep",
    field: "sleep_score",
  },
  {
    value: "sleep_duration",
    label: "Durée sommeil",
    unit: "min",
    source: "sleep",
    field: "duration_total",
  },
  { value: "steps", label: "Pas", unit: "pas", source: "activity", field: "steps" },
  {
    value: "calories_active",
    label: "Calories actives",
    unit: "kcal",
    source: "activity",
    field: "calories_active",
  },
  { value: "distance", label: "Distance", unit: "m", source: "activity", field: "distance" },
];

export const PRESETS = [
  { label: "Sommeil vs Activité", metric1: "sleep_score", metric2: "steps" },
  { label: "Poids vs Calories", metric1: "weight", metric2: "calories_active" },
  { label: "Tension vs Exercice", metric1: "systolic", metric2: "steps" },
];

export function getMetricOption(value: string): MetricOption {
  return METRIC_OPTIONS.find((m) => m.value === value) ?? FALLBACK_METRIC;
}

export function getDateKey(record: { measured_at?: string; date?: string }): string {
  if (record.date) return record.date.substring(0, 10);
  if (record.measured_at) return record.measured_at.substring(0, 10);
  return "";
}

export function computeCorrelation(data: { v1: number | null; v2: number | null }[]): {
  r: number;
  direction: string;
  strength: string;
} {
  const valid = data.filter(
    (d) => d.v1 != null && d.v2 != null && !Number.isNaN(d.v1) && !Number.isNaN(d.v2)
  ) as { v1: number; v2: number }[];

  if (valid.length < 3) return { r: 0, direction: "insuffisant", strength: "insuffisant" };

  const n = valid.length;
  const sumX = valid.reduce((s, d) => s + d.v1, 0);
  const sumY = valid.reduce((s, d) => s + d.v2, 0);
  const sumXY = valid.reduce((s, d) => s + d.v1 * d.v2, 0);
  const sumX2 = valid.reduce((s, d) => s + d.v1 * d.v1, 0);
  const sumY2 = valid.reduce((s, d) => s + d.v2 * d.v2, 0);

  const denom = Math.sqrt((n * sumX2 - sumX * sumX) * (n * sumY2 - sumY * sumY));

  if (denom === 0) return { r: 0, direction: "aucune", strength: "aucune" };

  const r = (n * sumXY - sumX * sumY) / denom;
  const absR = Math.abs(r);

  let direction = "aucune";
  if (r > 0.1) direction = "positive";
  else if (r < -0.1) direction = "negative";

  let strength = "aucune";
  if (absR >= 0.7) strength = "forte";
  else if (absR >= 0.4) strength = "modérée";
  else if (absR >= 0.2) strength = "faible";

  return { r, direction, strength };
}
