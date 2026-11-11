import { describe, expect, it } from "vitest";
import { aggregateBars, BAR_MAX_COUNT, periodLabel } from "./aggregateBars";

function days(n: number, start = "2020-01-01") {
  const t0 = Date.parse(`${start}T00:00:00Z`);
  return Array.from({ length: n }, (_, i) => ({
    date: new Date(t0 + i * 86_400_000).toISOString().slice(0, 10),
    steps: 1000 + i,
  }));
}

describe("aggregateBars", () => {
  it("garde une barre par jour jusqu'au plafond", () => {
    const rows = days(BAR_MAX_COUNT);
    const out = aggregateBars(rows, ["steps"]);
    expect(out.granularity).toBe("day");
    expect(out.rows).toBe(rows);
  });

  it("passe à la semaine au-dessus du plafond quand les semaines tiennent", () => {
    const out = aggregateBars(days(365), ["steps"]);
    expect(out.granularity).toBe("week");
    expect(out.rows.length).toBeLessThanOrEqual(BAR_MAX_COUNT);
    expect(out.rows.length).toBeGreaterThan(50);
  });

  it("passe au mois quand les semaines dépassent encore le plafond", () => {
    const out = aggregateBars(days(3650), ["steps"]);
    expect(out.granularity).toBe("month");
    expect(out.rows.length).toBeLessThanOrEqual(BAR_MAX_COUNT);
    expect(out.rows.length).toBeGreaterThan(100);
  });

  it("date chaque semaine de son lundi et chaque mois de son premier jour", () => {
    const week = aggregateBars(days(300, "2020-01-01"), ["steps"]);
    expect(week.rows[0]?.date).toBe("2019-12-30");
    const month = aggregateBars(days(3650, "2020-01-15"), ["steps"]);
    expect(month.rows[0]?.date).toBe("2020-01-01");
  });

  it("remplace chaque période par la moyenne de ses jours", () => {
    const out = aggregateBars(days(300, "2020-01-06"), ["steps"]);
    const first = out.rows[0];
    expect(first?.date).toBe("2020-01-06");
    expect(first?.steps).toBe((1000 + 1001 + 1002 + 1003 + 1004 + 1005 + 1006) / 7);
  });

  it("ignore les valeurs absentes dans la moyenne et rend null sans aucune valeur", () => {
    const rows = days(300).map((r, i) => ({ ...r, deep: i < 7 ? null : 2 }));
    const out = aggregateBars(rows, ["deep"]);
    const nullWeek = out.rows.find((r) => r.deep === null);
    expect(nullWeek).toBeDefined();
    expect(out.rows[out.rows.length - 1]?.deep).toBe(2);
  });

  it("n'altère pas la somme pondérée des jours", () => {
    const rows = days(400);
    const out = aggregateBars(rows, ["steps"]);
    const total = rows.reduce((a, r) => a + r.steps, 0);
    const rebuilt = out.rows.reduce((a, r) => a + r.steps * r.count, 0);
    expect(rebuilt).toBeCloseTo(total, 6);
  });

  it("trie par date avant de regrouper", () => {
    const rows = days(300).reverse();
    const out = aggregateBars(rows, ["steps"]);
    expect(out.rows[0]?.date).toBe("2019-12-30");
  });
});

describe("periodLabel", () => {
  it("nomme le jour, la semaine et le mois", () => {
    expect(periodLabel("day", "2020-03-04")).toBe("4 mars 2020");
    expect(periodLabel("week", "2020-03-02")).toBe("Semaine du 2 mars 2020");
    expect(periodLabel("month", "2020-03-01")).toBe("mars 2020");
  });
});
