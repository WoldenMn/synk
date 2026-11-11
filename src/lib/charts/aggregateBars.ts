import { formatDate } from "@/lib/formatters";

export const BAR_MAX_COUNT = 150;

export type BarGranularity = "day" | "week" | "month";

type Row = { date: string } & Record<string, string | number | null | undefined>;

export type BarPoint<T> = T & { count: number };

function weekStart(date: string): string {
  const d = new Date(`${date}T00:00:00Z`);
  const sinceMonday = (d.getUTCDay() + 6) % 7;
  d.setUTCDate(d.getUTCDate() - sinceMonday);
  const month = String(d.getUTCMonth() + 1).padStart(2, "0");
  const day = String(d.getUTCDate()).padStart(2, "0");
  return `${d.getUTCFullYear()}-${month}-${day}`;
}

function monthStart(date: string): string {
  return `${date.slice(0, 7)}-01`;
}

function bucketKey(granularity: BarGranularity, date: string): string {
  if (granularity === "week") return weekStart(date);
  if (granularity === "month") return monthStart(date);
  return date;
}

function distinctBuckets(rows: readonly Row[], granularity: BarGranularity): number {
  return new Set(rows.map((r) => bucketKey(granularity, r.date))).size;
}

export function aggregateBars<T extends Row>(
  rows: T[],
  keys: readonly (keyof T & string)[],
  max = BAR_MAX_COUNT
): { rows: BarPoint<T>[]; granularity: BarGranularity } {
  if (rows.length <= max) {
    return { rows: rows as BarPoint<T>[], granularity: "day" };
  }
  const sorted = [...rows].sort((a, b) => a.date.localeCompare(b.date));
  const granularity: BarGranularity = distinctBuckets(sorted, "week") <= max ? "week" : "month";
  const buckets = new Map<string, T[]>();
  for (const row of sorted) {
    const key = bucketKey(granularity, row.date);
    const bucket = buckets.get(key);
    if (bucket) bucket.push(row);
    else buckets.set(key, [row]);
  }
  const out: BarPoint<T>[] = [];
  for (const [date, bucket] of buckets) {
    const [first] = bucket;
    if (first === undefined) continue;
    const point: Record<string, unknown> = { ...first, date, count: bucket.length };
    for (const key of keys) {
      const values: number[] = [];
      for (const row of bucket) {
        const v: unknown = row[key];
        if (typeof v === "number") values.push(v);
      }
      point[key] = values.length > 0 ? values.reduce((a, v) => a + v, 0) / values.length : null;
    }
    out.push(point as BarPoint<T>);
  }
  return { rows: out, granularity };
}

export function periodLabel(granularity: BarGranularity, date: string): string {
  if (granularity === "week") return `Semaine du ${formatDate(date)}`;
  if (granularity === "month") return formatDate(date).replace(/^\d+ /, "");
  return formatDate(date);
}

export function aggregationLabelSuffix(granularity: BarGranularity): string {
  if (granularity === "week") return ", affichées en moyennes par semaine";
  if (granularity === "month") return ", affichées en moyennes par mois";
  return "";
}

export function aggregationNote(
  granularity: BarGranularity,
  shown: number,
  total: number,
  noun: string
): string | null {
  if (granularity === "week") {
    return `${shown} semaines affichées pour ${total} ${noun} (moyennes par semaine)`;
  }
  if (granularity === "month") {
    return `${shown} mois affichés pour ${total} ${noun} (moyennes par mois)`;
  }
  return null;
}
