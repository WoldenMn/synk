export interface BodyMeasure {
  id: number;
  measured_at: string;
  source: string;
  weight: number | null;
  fat_ratio: number | null;
  fat_mass: number | null;
  fat_free_mass: number | null;
  muscle_mass: number | null;
  bone_mass: number | null;
  hydration: number | null;
  bmi: number | null;
}

export interface ManualBodyMeasureInput {
  measured_at: string;
  weight: number | null;
  fat_ratio: number | null;
  fat_mass: number | null;
  muscle_mass: number | null;
  bone_mass: number | null;
  hydration: number | null;
}
