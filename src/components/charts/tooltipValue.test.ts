import { describe, expect, it } from "vitest";
import { tooltipNumber } from "./tooltipValue";

describe("tooltipNumber", () => {
  it("rend le nombre d'une valeur numérique ou numérique en texte", () => {
    expect(tooltipNumber(72.4)).toBe(72.4);
    expect(tooltipNumber("72.4")).toBe(72.4);
  });

  it("rend null pour une valeur absente ou non numérique", () => {
    expect(tooltipNumber(undefined)).toBeNull();
    expect(tooltipNumber(Number.NaN)).toBeNull();
    expect(tooltipNumber([1, 2])).toBeNull();
    expect(tooltipNumber("abc")).toBeNull();
  });
});
