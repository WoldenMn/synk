import { invoke } from "@tauri-apps/api/core";
import type { ClothingInputs, ClothingSize } from "./types";

export function getClothingInputs(): Promise<ClothingInputs> {
  return invoke<ClothingInputs>("get_clothing_inputs");
}

export function getClothingSizes(): Promise<ClothingSize[]> {
  return invoke<ClothingSize[]>("get_clothing_sizes");
}

export function setClothingSize(
  category: string,
  value: string,
  source: "override" | "manual"
): Promise<void> {
  return invoke("set_clothing_size", { category, value, source });
}

export function deleteClothingSize(category: string): Promise<void> {
  return invoke("delete_clothing_size", { category });
}

export function saveEstimatedSizes(sizes: { category: string; value: string }[]): Promise<void> {
  return invoke("save_estimated_sizes", { sizes });
}
