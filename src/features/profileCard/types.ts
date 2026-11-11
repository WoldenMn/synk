export interface Body {
  age: number | null;
  sex: string | null;
  height_cm: number | null;
  weight_kg: number | null;
  bmi: number | null;
  fat_ratio: number | null;
  muscle_mass: number | null;
  hydration: number | null;
  measured_at: string | null;
}

export interface KeyMeasurement {
  slug: string;
  label: string;
  value: number | null;
  measured_at: string | null;
}

export interface Health {
  active_treatments: string[];
}

export interface SizeEntry {
  category: string;
  value: string;
}

export interface RecordEntry {
  key: string;
  label: string;
  value: number;
  unit: string;
  achieved_at: string;
}

export interface ProfileCard {
  body: Body;
  key_measurements: KeyMeasurement[];
  health: Health;
  sizes: SizeEntry[];
  records: RecordEntry[];
}
