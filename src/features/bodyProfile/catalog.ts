import raw from "./catalog.json";
import type { BodyMeasurement, Zone } from "./types";

// Double cast requis : TypeScript infere `string` pour les champs du JSON, ce qui
// n'est pas assignable directement aux unions litterales de BodyMeasurement.
export const CATALOG = raw.measurements as unknown as BodyMeasurement[];

export const ZONES: { id: Zone; label: string }[] = [
  { id: "global", label: "Global" },
  { id: "neck_shoulders", label: "Cou et épaules" },
  { id: "torso", label: "Tronc" },
  { id: "arms", label: "Bras" },
  { id: "legs", label: "Jambes" },
];

export const KEY_MEASUREMENTS = CATALOG.filter((m) => m.isKey);

export function findMeasurement(slug: string): BodyMeasurement | undefined {
  return CATALOG.find((m) => m.slug === slug);
}

export function byZone(): { zone: Zone; label: string; measurements: BodyMeasurement[] }[] {
  return ZONES.map((z) => ({
    zone: z.id,
    label: z.label,
    measurements: CATALOG.filter((m) => m.zone === z.id).sort((a, b) => a.order - b.order),
  }));
}
