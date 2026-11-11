export interface HealthAlert {
  id: number;
  metric_key: string;
  condition: string;
  threshold: number;
  label: string;
  enabled: boolean;
  created_at: string;
}

export interface AlertEvent {
  id: number;
  alert_id: number;
  value: number;
  triggered_at: string;
  seen: boolean;
}

export interface AddAlertInput {
  metric_key: string;
  condition: string;
  threshold: number;
  label: string;
}
