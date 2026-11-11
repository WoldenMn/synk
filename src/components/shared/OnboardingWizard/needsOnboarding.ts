const MEASURE_COUNTS = [
  "body_measures_count",
  "blood_pressure_count",
  "sleep_count",
  "activity_count",
  "workouts_count",
] as const;

function measureCount(metrics: unknown): number {
  if (typeof metrics !== "object" || metrics === null) {
    throw new Error("métriques de la base illisibles");
  }
  const record = metrics as Record<string, unknown>;
  return MEASURE_COUNTS.reduce((sum, key) => {
    const value = record[key];
    if (typeof value !== "number" || !Number.isFinite(value)) {
      throw new Error(`compteur ${key} illisible`);
    }
    return sum + value;
  }, 0);
}

export function needsOnboarding(metrics: unknown, sources: unknown): boolean {
  if (!Array.isArray(sources)) throw new Error("liste des sources illisible");
  const connected = sources.some(
    (s: unknown) =>
      typeof s === "object" && s !== null && (s as { connected?: unknown }).connected === true
  );
  return measureCount(metrics) === 0 && !connected;
}
