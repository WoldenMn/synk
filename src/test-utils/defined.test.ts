import { describe, expect, it } from "vitest";
import { defined } from "./defined";

describe("defined", () => {
  it("rend la valeur présente, zéro et chaîne vide compris", () => {
    expect(defined(0)).toBe(0);
    expect(defined("")).toBe("");
    expect(defined([1][0])).toBe(1);
  });

  it("échoue sur undefined et sur null en nommant la valeur", () => {
    expect(() => defined(undefined, "le premier appel")).toThrow(/le premier appel absente/);
    expect(() => defined(null)).toThrow(/valeur absente/);
  });
});
