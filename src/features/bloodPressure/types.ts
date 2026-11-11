export interface BloodPressure {
  id: number;
  measured_at: string;
  source: string;
  systolic: number;
  diastolic: number;
  heart_rate: number | null;
}

export interface ManualBloodPressureInput {
  measured_at: string;
  systolic: number;
  diastolic: number;
  heart_rate: number | null;
}
