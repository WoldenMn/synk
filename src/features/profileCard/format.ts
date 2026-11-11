import { formatDate, formatDuration, formatNumber } from "@/lib/formatters";

const DECIMALS: Record<string, number> = { kg: 1, "%": 1, cm: 1 };

/** Valeur et unite d'une mesure du profil : « 76,0 kg », « 21,6 % », « 180 cm ». */
export function formatMeasure(value: number | null | undefined, unit: string): string {
  if (value == null || Number.isNaN(value)) return "—";
  return `${formatNumber(value, DECIMALS[unit] ?? 0)} ${unit}`;
}

/**
 * Valeur d'un record. Le backend envoie les durees en secondes (unite « s »,
 * comme les tables sleep_records et workouts), affichees en heures et minutes.
 */
export function formatRecordValue(value: number, unit: string): string {
  if (unit === "s") return formatDuration(value);
  if (unit === "/100") return `${formatNumber(value, 0)}/100`;
  return formatMeasure(value, unit);
}

export function formatProfileDate(iso: string | null | undefined): string {
  return iso ? formatDate(iso) : "—";
}
