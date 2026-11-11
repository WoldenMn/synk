import type { ImportReport } from "./types";

function count(n: number, one: string, many: string): string | null {
  if (n <= 0) return null;
  return `${n} ${n === 1 ? one : many}`;
}

function tallyParts(report: ImportReport): string[] {
  const t = report.tally;
  return [
    count(t.observations, "mesure", "mesures"),
    count(t.blood_pressure, "tension", "tensions"),
    count(t.sleep, "nuit", "nuits"),
    count(t.activity, "jour d'activité", "jours d'activité"),
    count(t.workouts, "séance", "séances"),
  ].filter((p): p is string => p !== null);
}

export function summarizeImportReport(report: ImportReport): string {
  const parts = tallyParts(report);
  return parts.length > 0 ? parts.join(", ") : "Rien de nouveau, tout était déjà importé";
}

export function importToastText(report: ImportReport): string {
  const parts = tallyParts(report);
  return parts.length > 0
    ? `Import terminé : ${parts.join(", ")}`
    : "Import terminé, mais rien de nouveau : tout était déjà dans Synk";
}

export function skippedLines(report: ImportReport): string[] {
  return report.skipped.map((s) => `${s.reason} (${s.count})`);
}
