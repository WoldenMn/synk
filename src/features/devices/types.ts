export interface WithingsDevice {
  id: number;
  device_id: string;
  device_type: string | null;
  model: string | null;
  model_id: number | null;
  battery: string | null;
  last_session_date: string | null;
  first_session_date: string | null;
}
