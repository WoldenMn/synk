import { describe, expect, it } from "vitest";
import { chartDensity, decimate } from "./decimate";

const series = (n: number) => Array.from({ length: n }, (_, i) => ({ date: `j${i}`, value: i }));

describe("decimate", () => {
  it("rend la série telle quelle sous le plafond", () => {
    const s = series(120);
    expect(decimate(s, 500, ["value"])).toBe(s);
  });

  it("plafonne, garde le premier et le dernier point", () => {
    const out = decimate(series(2300), 500, ["value"]);
    expect(out.length).toBeLessThanOrEqual(500);
    expect(out[0]).toEqual({ date: "j0", value: 0 });
    expect(out[out.length - 1]).toEqual({ date: "j2299", value: 2299 });
  });

  it("remplace chaque seau par sa moyenne", () => {
    const out = decimate(series(12), 5, ["value"]);
    expect(out.map((p) => p.value)).toEqual([0, 2, 5, 8.5, 11]);
  });

  it("ignore les valeurs absentes dans la moyenne", () => {
    const s = [
      { date: "a", value: 1 as number | null },
      { date: "b", value: null },
      { date: "c", value: 3 },
      { date: "d", value: 5 },
      { date: "e", value: 7 },
    ];
    expect(decimate(s, 3, ["value"]).map((p) => p.value)).toEqual([1, 4, 7]);
  });
});

describe("chartDensity", () => {
  it("cercles jusqu'à 90 points, animation jusqu'à 200", () => {
    expect(chartDensity(90)).toEqual({ showDots: true, animate: true });
    expect(chartDensity(91)).toEqual({ showDots: false, animate: true });
    expect(chartDensity(201)).toEqual({ showDots: false, animate: false });
  });
});
