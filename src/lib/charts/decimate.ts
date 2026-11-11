export const CHART_MAX_POINTS = 500;

type Row = Record<string, string | number | null | undefined>;

export function decimate<T extends Row>(
  points: T[],
  max: number,
  keys: readonly (keyof T & string)[]
): T[] {
  if (points.length <= max || max < 3) return points;
  const first = points[0];
  const last = points[points.length - 1];
  if (first === undefined || last === undefined) return points;
  const inner = points.slice(1, -1);
  const buckets = max - 2;
  const size = inner.length / buckets;
  const out: T[] = [first];
  for (let b = 0; b < buckets; b++) {
    const bucket = inner.slice(Math.floor(b * size), Math.floor((b + 1) * size));
    if (bucket.length === 0) continue;
    const row: Row = { ...bucket[Math.floor(bucket.length / 2)] };
    for (const key of keys) {
      const values: number[] = [];
      for (const p of bucket) {
        const v: unknown = p[key];
        if (typeof v === "number") values.push(v);
      }
      row[key] = values.length > 0 ? values.reduce((a, v) => a + v, 0) / values.length : null;
    }
    out.push(row as T);
  }
  out.push(last);
  return out;
}

export function chartDensity(n: number): { showDots: boolean; animate: boolean } {
  return { showDots: n <= 90, animate: n <= 200 };
}
