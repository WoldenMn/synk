import { invoke } from "@tauri-apps/api/core";
import type { ProfileCard } from "./types";

export function getProfileCard(): Promise<ProfileCard> {
  return invoke<ProfileCard>("get_profile_card");
}

export function exportProfileCardPdf(): Promise<string> {
  return invoke<string>("export_profile_card_pdf");
}
