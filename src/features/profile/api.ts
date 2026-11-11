import { invoke } from "@tauri-apps/api/core";
import type { UserProfile } from "@/features/profile/types";

export function getUserProfile(): Promise<UserProfile> {
  return invoke<UserProfile>("get_user_profile");
}

export function updateUserProfile(
  age: number | null,
  sex: string | null,
  heightCm: number | null
): Promise<void> {
  return invoke("update_user_profile", { age, sex, heightCm });
}
