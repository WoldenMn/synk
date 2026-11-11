export type Zone = "global" | "neck_shoulders" | "torso" | "arms" | "legs";
export type Nature = "circumference" | "length" | "width" | "height";
export type Side = "none" | "left" | "right";

export interface Anchor {
  view: "front" | "back";
  x: number;
  y: number;
  labelSide: "left" | "right";
}

export interface BodyMeasurement {
  slug: string;
  label: string;
  stepLabel: string;
  zone: Zone;
  nature: Nature;
  side: Side;
  pairKey: string | null;
  order: number;
  isKey: boolean;
  min: number;
  max: number;
  guidance: string;
  anchor: Anchor | null;
}

export interface SessionEntry {
  slug: string;
  value: number;
}

export interface SheetRow {
  slug: string;
  label: string;
  unit: string;
  value: number | null;
  measured_at: string | null;
}
