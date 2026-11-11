export interface Activity {
  id: number;
  date: string;
  source: string;
  steps: number | null;
  distance: number | null;
  elevation: number | null;
  calories_active: number | null;
  calories_total: number | null;
  duration_soft: number | null;
  duration_moderate: number | null;
  duration_intense: number | null;
  duration_active: number | null;
  hr_average: number | null;
  hr_min: number | null;
  hr_max: number | null;
  hr_zone_0: number | null;
  hr_zone_1: number | null;
  hr_zone_2: number | null;
  hr_zone_3: number | null;
}

export interface ManualActivityInput {
  date: string;
  steps: number | null;
  distance: number | null;
  calories_active: number | null;
  duration_active: number | null;
}
