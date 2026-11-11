import { describe, expect, it } from "vitest";
import {
  columnMetric,
  columnUnit,
  importIssue,
  pressureIssue,
  setColumnMetric,
  setColumnUnit,
} from "./csvConfig";
import type { CsvConfig, CsvTarget } from "./types";

const TARGETS: CsvTarget[] = [
  {
    metric: "weight",
    label: "Poids",
    units: [
      { id: "kg", label: "kg" },
      { id: "lb", label: "lb" },
    ],
  },
  { metric: "systolic", label: "Tension systolique", units: [{ id: "mmHg", label: "mmHg" }] },
  { metric: "diastolic", label: "Tension diastolique", units: [{ id: "mmHg", label: "mmHg" }] },
];

const BASE: CsvConfig = {
  delimiter: ";",
  decimal: ",",
  skip_rows: 0,
  date_column: "Date",
  time_column: null,
  date_format: "iso",
  columns: [],
};

describe("csvConfig", () => {
  it("associe une colonne à une mesure avec la première unité", () => {
    const next = setColumnMetric(BASE, "Poids", "weight", TARGETS);
    expect(columnMetric(next, "Poids")).toBe("weight");
    expect(columnUnit(next, "Poids")).toBe("kg");
  });

  it("garde l'unité choisie quand la mesure change dans la même famille, sinon reprend la première", () => {
    let c = setColumnMetric(BASE, "Poids", "weight", TARGETS);
    c = setColumnUnit(c, "Poids", "lb");
    expect(columnUnit(setColumnMetric(c, "Poids", "weight", TARGETS), "Poids")).toBe("lb");
    expect(columnUnit(setColumnMetric(c, "Poids", "systolic", TARGETS), "Poids")).toBe("mmHg");
  });

  it("retire la colonne quand on choisit « ignorer »", () => {
    const c = setColumnMetric(BASE, "Poids", "weight", TARGETS);
    expect(setColumnMetric(c, "Poids", "", TARGETS).columns).toEqual([]);
  });

  it("refuse une tension incomplète et une configuration vide", () => {
    const half = setColumnMetric(BASE, "Sys", "systolic", TARGETS);
    expect(pressureIssue(half)).toMatch(/systolique et une colonne diastolique/);
    const full = setColumnMetric(half, "Dia", "diastolic", TARGETS);
    expect(pressureIssue(full)).toBeNull();
    expect(importIssue(BASE)).toMatch(/au moins une colonne/);
    expect(importIssue({ ...full, date_column: "" })).toMatch(/date/);
    expect(importIssue(full)).toBeNull();
  });
});
