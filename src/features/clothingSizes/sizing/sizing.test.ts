import { describe, it, expect } from "vitest";
import {
  estimateJacket,
  estimateShirt,
  estimateTop,
  estimatePants,
  computeSizes,
  shoeConversion,
  resolveBandIndex,
} from "./index";

describe("estimateJacket", () => {
  it("poitrine au centre d'un palier -> taille sans alerte", () => {
    const r = estimateJacket(100);
    expect(r?.eu).toBe("50");
    expect(r?.nearThreshold).toBeUndefined();
  });
  it("poitrine en limite haute -> alerte vers le dessus", () => {
    const r = estimateJacket(98);
    expect(r?.eu).toBe("48");
    expect(r?.nearThreshold).toEqual({ size: "50", direction: "up" });
  });
  it("poitrine en limite basse -> alerte vers le dessous", () => {
    const r = estimateJacket(91);
    expect(r?.eu).toBe("46");
    expect(r?.nearThreshold).toEqual({ size: "44", direction: "down" });
  });
  it("hors borne -> null", () => {
    expect(estimateJacket(10)).toBeNull();
    expect(estimateJacket(null)).toBeNull();
  });
});

describe("estimateShirt", () => {
  it("cou 40 -> col 40", () => {
    expect(estimateShirt(40)?.eu).toBe("40");
  });
  it("cou manquant -> null", () => {
    expect(estimateShirt(null)).toBeNull();
  });
});

describe("estimateTop", () => {
  it("poitrine 98 -> M (alignée sur les chartes EU)", () => {
    expect(estimateTop(98)?.eu).toBe("M");
  });
  it("retourne une taille lettre pour une poitrine valide", () => {
    expect(estimateTop(110)?.eu).toBeTypeOf("string");
  });
});

describe("estimatePants", () => {
  it("ajoute la longueur L quand l'entrejambe est fourni", () => {
    const r = estimatePants(84, 84);
    expect(r?.us).toMatch(/^W\d+ L\d+$/);
  });
  it("sans entrejambe -> W seul", () => {
    const r = estimatePants(84, null);
    expect(r?.us).toMatch(/^W\d+$/);
  });
});

describe("computeSizes", () => {
  it("mappe chaque catégorie calculable", () => {
    const r = computeSizes({ chest: 98, neck: 40, waist: 84, inseam: 84 });
    expect(r.jacket?.eu).toBeTypeOf("string");
    expect(r.shirt?.eu).toBe("40");
    expect(r.top?.eu).toBe("M");
    expect(r.pants?.eu).toBeTypeOf("string");
  });
  it("mensuration manquante -> catégorie null", () => {
    const r = computeSizes({ chest: null, neck: null, waist: null, inseam: null });
    expect(r.jacket).toBeNull();
    expect(r.shirt).toBeNull();
  });
});

describe("shoeConversion", () => {
  it("convertit une pointure EU connue", () => {
    expect(shoeConversion("43")).toEqual({ us: "9.5", uk: "9" });
  });
  it("pointure inconnue -> undefined", () => {
    expect(shoeConversion("99")).toBeUndefined();
  });
});

describe("resolveBandIndex", () => {
  const opts = { min: 0, max: 200, margin: 2 };
  const bands = [{ maxValue: 90 }, { maxValue: 100 }, { maxValue: 110 }];

  it("une liste de paliers vide ne donne rien au lieu de lever une erreur", () => {
    expect(resolveBandIndex(95, [], opts)).toBeNull();
  });

  it("rend le palier retenu et le palier voisin quand le seuil est proche", () => {
    const res = resolveBandIndex(99, bands, opts);
    expect(res?.idx).toBe(1);
    expect(res?.band).toBe(bands[1]);
    expect(res?.near).toEqual({ idx: 2, band: bands[2], direction: "up" });
    const low = resolveBandIndex(91, bands, opts);
    expect(low?.near).toEqual({ idx: 0, band: bands[0], direction: "down" });
  });

  it("sans seuil proche, pas de voisin", () => {
    expect(resolveBandIndex(95, bands, opts)?.near).toBeNull();
  });
});
