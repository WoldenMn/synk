export function avg<T>(records: T[], key: keyof T): number | null {
  const valid = records.filter((r) => r[key] != null);
  if (valid.length === 0) return null;
  return valid.reduce((sum, r) => sum + ((r[key] as number) || 0), 0) / valid.length;
}

export function sum<T>(records: T[], key: keyof T): number {
  return records.reduce((total, r) => total + ((r[key] as number) || 0), 0);
}

export function latest<T>(records: T[], key: keyof T): number | null {
  if (records.length === 0) return null;
  const last = records[records.length - 1];
  if (last == null) return null;
  return (last[key] as number) ?? null;
}
