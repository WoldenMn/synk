import { format, parseISO } from "date-fns";
import { fr } from "date-fns/locale";

export function formatDate(dateStr: string): string {
  if (!dateStr) return "\u2014";
  return format(parseISO(dateStr), "d MMM yyyy", { locale: fr });
}

export function formatDateTime(dateStr: string): string {
  if (!dateStr) return "\u2014";
  return format(parseISO(dateStr), "d MMM yyyy HH:mm", { locale: fr });
}

export function formatNumber(n: number, decimals = 1): string {
  if (n == null || Number.isNaN(n)) return "\u2014";
  return n.toLocaleString("fr-FR", {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}

export function formatWeight(kg: number): string {
  if (kg == null) return "\u2014";
  return `${formatNumber(kg)} kg`;
}

export function formatPercent(pct: number): string {
  if (pct == null) return "\u2014";
  return `${formatNumber(pct)} %`;
}

export function formatDuration(seconds: number): string {
  if (seconds == null || seconds < 0) return "\u2014";
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours === 0) return `${minutes}min`;
  return `${hours}h ${minutes.toString().padStart(2, "0")}min`;
}

export function formatBP(sys: number, dia: number): string {
  if (sys == null || dia == null) return "\u2014";
  return `${sys}/${dia}`;
}

export function orDash<T>(value: T | null | undefined, format: (v: T) => string): string {
  return value == null ? "\u2014" : format(value);
}
