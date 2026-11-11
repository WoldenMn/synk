export const METRIC_OPTIONS = [
  { value: "weight", label: "Poids (kg)" },
  { value: "fat_ratio", label: "Masse grasse (%)" },
  { value: "muscle_mass", label: "Masse musculaire (kg)" },
  { value: "bmi", label: "IMC" },
  { value: "systolic", label: "Systolique (mmHg)" },
  { value: "diastolic", label: "Diastolique (mmHg)" },
  { value: "heart_rate", label: "Fréquence cardiaque (bpm)" },
  { value: "sleep_score", label: "Score sommeil" },
  { value: "steps", label: "Pas" },
];

export const CONDITION_OPTIONS = [
  { value: "above", label: "Au-dessus de" },
  { value: "below", label: "En-dessous de" },
];

export function getMetricLabel(key: string): string {
  return METRIC_OPTIONS.find((m) => m.value === key)?.label ?? key;
}
