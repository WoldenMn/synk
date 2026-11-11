export interface LabResult {
  id: number;
  date: string;
  test_name: string;
  value: number;
  unit: string | null;
  reference_min: number | null;
  reference_max: number | null;
  notes: string | null;
  created_at: string;
}

export interface AddLabResultInput {
  date: string;
  test_name: string;
  value: number;
  unit: string | null;
  reference_min: number | null;
  reference_max: number | null;
  notes: string | null;
}
