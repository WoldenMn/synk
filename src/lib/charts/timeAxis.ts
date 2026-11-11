import { formatDate } from "@/lib/formatters";

export const TIME_AXIS = {
  dataKey: "ts",
  type: "number",
  scale: "time",
  domain: ["dataMin", "dataMax"],
} as const;

export function distinctTicks(rows: readonly { ts: number }[]): number[] {
  return Array.from(new Set(rows.map((row) => row.ts)));
}

export function timeAxisFor(rows: readonly { ts: number }[]) {
  return { ...TIME_AXIS, ticks: distinctTicks(rows) };
}

export function withTimestamp<T extends { date: string }>(rows: T[]): (T & { ts: number })[] {
  return rows
    .map((row) => ({ ...row, ts: Date.parse(row.date.slice(0, 10)) }))
    .filter((row) => Number.isFinite(row.ts))
    .sort((a, b) => a.ts - b.ts);
}

export function tickDay(ts: number): string {
  const d = new Date(ts);
  const month = String(d.getUTCMonth() + 1).padStart(2, "0");
  const day = String(d.getUTCDate()).padStart(2, "0");
  return `${d.getUTCFullYear()}-${month}-${day}`;
}

export function formatTick(ts: number): string {
  return formatDate(tickDay(ts));
}
