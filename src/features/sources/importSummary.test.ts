import { describe, expect, it } from "vitest";
import { importToastText, skippedLines, summarizeImportReport } from "./importSummary";
import type { ImportReport } from "./types";

const REPORT: ImportReport = {
  source: "apple_health",
  cancelled: false,
  tally: {
    observations: 12,
    observations_ignored: 3,
    blood_pressure: 1,
    sleep: 2,
    activity: 30,
    workouts: 0,
    devices: { Withings: 5, "Apple Watch": 40 },
  },
  skipped: [
    { reason: "unité inconnue : furlong", count: 2 },
    { reason: "tension : une seule valeur sur deux", count: 1 },
  ],
  duration_ms: 1500,
};

describe("summarizeImportReport", () => {
  it("compte chaque famille écrite et passe sous silence celles à zéro", () => {
    expect(summarizeImportReport(REPORT)).toBe(
      "12 mesures, 1 tension, 2 nuits, 30 jours d'activité"
    );
  });

  it("dit qu'il n'y a rien de nouveau quand tout existait déjà", () => {
    const again = {
      ...REPORT,
      tally: { ...REPORT.tally, observations: 0, blood_pressure: 0, sleep: 0, activity: 0 },
    };
    expect(summarizeImportReport(again)).toBe("Rien de nouveau, tout était déjà importé");
  });

  it("annonce la fin de l'import sans empiler deux fois deux-points", () => {
    expect(importToastText(REPORT)).toBe(
      "Import terminé : 12 mesures, 1 tension, 2 nuits, 30 jours d'activité"
    );
    const again = {
      ...REPORT,
      tally: { ...REPORT.tally, observations: 0, blood_pressure: 0, sleep: 0, activity: 0 },
    };
    const text = importToastText(again);
    expect(text).toBe("Import terminé, mais rien de nouveau : tout était déjà dans Synk");
    expect(text.match(/:/g)).toHaveLength(1);
    expect(text).not.toContain("—");
  });

  it("accorde le singulier", () => {
    const one = {
      ...REPORT,
      tally: {
        ...REPORT.tally,
        observations: 1,
        blood_pressure: 0,
        sleep: 1,
        activity: 1,
        workouts: 1,
      },
    };
    expect(summarizeImportReport(one)).toBe("1 mesure, 1 nuit, 1 jour d'activité, 1 séance");
  });
});

describe("skippedLines", () => {
  it("liste les lignes ignorées avec leur nombre", () => {
    expect(skippedLines(REPORT)).toEqual([
      "unité inconnue : furlong (2)",
      "tension : une seule valeur sur deux (1)",
    ]);
  });
});
