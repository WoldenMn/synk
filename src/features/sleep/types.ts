export interface SleepRecord {
  id: number;
  date: string;
  source: string;
  start_at: string | null;
  end_at: string | null;
  duration_total: number | null;
  duration_deep: number | null;
  duration_light: number | null;
  duration_rem: number | null;
  duration_awake: number | null;
  duration_to_sleep: number | null;
  duration_to_wakeup: number | null;
  sleep_score: number | null;
  hr_average: number | null;
  hr_min: number | null;
  hr_max: number | null;
  rr_average: number | null;
  snoring: number | null;
  snoring_count: number | null;
  sleep_efficiency: number | null;
  nb_rem_episodes: number | null;
  out_of_bed_count: number | null;
  waso: number | null;
  total_timeinbed: number | null;
  apnea_hypopnea_index: number | null;
  breathing_disturbances_intensity: number | null;
  asleepduration: number | null;
  night_events: number | null;
}

export interface ManualSleepInput {
  date: string;
  start_at: string | null;
  end_at: string | null;
  duration_total: number | null;
  sleep_score: number | null;
}
