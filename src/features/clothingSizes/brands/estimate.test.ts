import { describe, it, expect } from "vitest";
import { estimateForBrand } from "./estimate";
import type { Brand } from "./types";
import type { ClothingInputs } from "../types";

const FIXTURE: Brand[] = [
  {
    id: "fixture",
    label: "Fixture",
    grids: {
      top: [
        { maxValue: 96, label: "S" },
        { maxValue: 104, label: "M" },
        { maxValue: 112, label: "L" },
      ],
      pants: [
        { maxValue: 79, label: "30" },
        { maxValue: 84, label: "32" },
        { maxValue: 89, label: "34" },
      ],
    },
  },
];

const inputs = (over: Partial<ClothingInputs> = {}): ClothingInputs => ({
  chest: null,
  neck: null,
  waist: null,
  inseam: null,
  ...over,
});

describe("estimateForBrand", () => {
  it("poitrine au centre d'un palier -> label sans alerte", () => {
    const r = estimateForBrand("fixture", "top", inputs({ chest: 100 }), FIXTURE);
    expect(r?.label).toBe("M");
    expect(r?.nearThreshold).toBeUndefined();
  });

  it("poitrine proche du seuil haut -> alerte vers le dessus", () => {
    const r = estimateForBrand("fixture", "top", inputs({ chest: 103 }), FIXTURE);
    expect(r?.label).toBe("M");
    expect(r?.nearThreshold).toEqual({ size: "L", direction: "up" });
  });

  it("pantalon utilise le tour de taille", () => {
    const r = estimateForBrand("fixture", "pants", inputs({ waist: 82 }), FIXTURE);
    expect(r?.label).toBe("32");
  });

  it("catégorie absente de la marque -> null", () => {
    const r = estimateForBrand("fixture", "shirt", inputs({ neck: 40 }), FIXTURE);
    expect(r).toBeNull();
  });

  it("mesure manquante -> null", () => {
    const r = estimateForBrand("fixture", "top", inputs({ chest: null }), FIXTURE);
    expect(r).toBeNull();
  });

  it("marque inconnue -> null", () => {
    const r = estimateForBrand("inconnue", "top", inputs({ chest: 100 }), FIXTURE);
    expect(r).toBeNull();
  });
});
