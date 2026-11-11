export interface Medication {
  id: number;
  name: string;
  dosage: string | null;
  frequency: string | null;
  start_date: string | null;
  end_date: string | null;
  notes: string | null;
  active: boolean;
  created_at: string;
}

export interface AddMedicationInput {
  name: string;
  dosage: string | null;
  frequency: string | null;
  start_date: string | null;
  end_date: string | null;
  notes: string | null;
}

export interface UpdateMedicationInput {
  id: number;
  name: string;
  dosage: string | null;
  frequency: string | null;
  start_date: string | null;
  end_date: string | null;
  notes: string | null;
  active: boolean;
}

export interface MedicationLog {
  id: number;
  medication_id: number;
  taken_at: string;
  skipped: boolean;
  notes: string | null;
  created_at: string;
}

export interface LogMedicationInput {
  medication_id: number;
  taken_at: string;
  skipped: boolean;
  notes: string | null;
}
