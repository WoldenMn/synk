export function tooltipNumber(value: unknown): number | null {
  const n =
    typeof value === "number" ? value : typeof value === "string" ? Number(value) : Number.NaN;
  return Number.isFinite(n) ? n : null;
}
