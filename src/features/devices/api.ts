import { invoke } from "@tauri-apps/api/core";
import type { WithingsDevice } from "./types";

export function getWithingsDevices(): Promise<WithingsDevice[]> {
  return invoke<WithingsDevice[]>("get_withings_devices");
}
