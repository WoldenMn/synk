import { invoke } from "@tauri-apps/api/core";
import type { HealthAlert, AlertEvent, AddAlertInput } from "./types";

export function getAlerts(): Promise<HealthAlert[]> {
  return invoke<HealthAlert[]>("get_alerts");
}

export function addAlert(data: AddAlertInput): Promise<number> {
  return invoke<number>("add_alert", { data });
}

export function deleteAlert(id: number): Promise<void> {
  return invoke("delete_alert", { id });
}

export function getAlertEvents(): Promise<AlertEvent[]> {
  return invoke<AlertEvent[]>("get_alert_events");
}

export function markAlertsSeen(): Promise<void> {
  return invoke("mark_alerts_seen");
}

export function checkAlerts(): Promise<AlertEvent[]> {
  return invoke<AlertEvent[]>("evaluate_alerts");
}
