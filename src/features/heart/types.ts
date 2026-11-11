export interface HeartRecord {
  id: number;
  timestamp: string;
  source: string;
  ecg_status: number | null;
  heart_rate: number | null;
  qrs_duration: number | null;
}
