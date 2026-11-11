export interface Workout {
  id: number;
  date: string;
  source: string;
  start_at: string | null;
  end_at: string | null;
  category: number | null;
  category_label: string | null;
  duration: number | null;
  pause_duration: number | null;
  distance: number | null;
  steps: number | null;
  elevation: number | null;
  calories: number | null;
  hr_average: number | null;
  hr_min: number | null;
  hr_max: number | null;
  hr_zone_0: number | null;
  hr_zone_1: number | null;
  hr_zone_2: number | null;
  hr_zone_3: number | null;
  spo2_average: number | null;
  intensity: number | null;
  effort_score: number | null;
  pool_laps: number | null;
  pool_length: number | null;
}

export interface ManualWorkoutInput {
  date: string;
  category_label: string | null;
  duration: number | null;
  distance: number | null;
  steps: number | null;
  calories: number | null;
  hr_average: number | null;
}
