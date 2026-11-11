export type SizeSystem = "eu" | "us" | "uk";

export type CategoryKey = "top" | "shirt" | "jacket" | "pants" | "shoes";

export interface ClothingInputs {
  chest: number | null;
  neck: number | null;
  waist: number | null;
  inseam: number | null;
}

export interface ClothingSize {
  category: string;
  value: string;
  source: "override" | "manual";
  updated_at: string;
}

export interface SizeEstimate {
  eu: string;
  us?: string;
  uk?: string;
  nearThreshold?: { size: string; direction: "down" | "up" };
}
