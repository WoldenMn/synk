import { describe, it, expect } from "vitest";
import { BRANDS } from "./index";
import { defined } from "@/test-utils/defined";

describe("intégrité des grilles de marque", () => {
  it("au moins une marque est définie", () => {
    expect(BRANDS.length).toBeGreaterThan(0);
  });

  it("chaque marque a un id et un label non vides et des id uniques", () => {
    const ids = new Set<string>();
    for (const b of BRANDS) {
      expect(b.id.trim()).not.toBe("");
      expect(b.label.trim()).not.toBe("");
      expect(ids.has(b.id)).toBe(false);
      ids.add(b.id);
    }
  });

  it("chaque grille est non vide, triée par maxValue croissant, labels non vides", () => {
    for (const b of BRANDS) {
      for (const [category, bands] of Object.entries(b.grids)) {
        expect(bands.length, `${b.id}/${category} vide`).toBeGreaterThan(0);
        for (let i = 1; i < bands.length; i++) {
          expect(
            defined(bands[i]).maxValue,
            `${b.id}/${category} non trié à l'index ${i}`
          ).toBeGreaterThan(defined(bands[i - 1]).maxValue);
        }
        for (const band of bands) {
          expect(band.label.trim(), `${b.id}/${category} label vide`).not.toBe("");
        }
      }
    }
  });
});
