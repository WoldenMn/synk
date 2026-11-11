interface PrimaryRow {
  date: string;
  value: number;
  source: string;
}

interface SecondaryPoint {
  measured_at: string;
  value: number;
  source: string;
}

export type MergedRow<T extends PrimaryRow> = T & { other?: number; otherSource?: string };

export function mergeSecondary<T extends PrimaryRow>(
  rows: T[],
  secondary: readonly SecondaryPoint[]
): MergedRow<T>[] {
  const byDay = new Map<string, SecondaryPoint>();
  for (const point of secondary) {
    const day = point.measured_at.slice(0, 10);
    if (!byDay.has(day) && Number.isFinite(point.value)) byDay.set(day, point);
  }
  return rows.map((row) => {
    const point = byDay.get(row.date.slice(0, 10));
    return point ? { ...row, other: point.value, otherSource: point.source } : row;
  });
}
