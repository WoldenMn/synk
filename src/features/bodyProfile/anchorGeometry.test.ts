import { describe, expect, it } from "vitest";
import { PLATE_HEIGHT, PLATE_LANDMARKS, PLATE_WIDTH, anchorToPlatePoint } from "./anchorGeometry";
import catalog from "./catalog.json";
import { defined } from "@/test-utils/defined";

// Chaque ancre annonce annoter un repere precis du trace. Sans ce verrou,
// retoucher la planche decale les pastilles de la fiche et du PDF sans que
// rien ne le signale — c'est exactement ce qui est arrive.
const SUR_UN_REPERE: [string, keyof typeof PLATE_LANDMARKS][] = [
  ["height", "crown"],
  ["neck", "neck"],
  ["shoulder_length_left", "shoulder"],
  ["chest", "chest"],
  ["waist", "waist"],
  ["hips", "hips"],
  ["inseam", "crotch"],
  ["thigh_left", "thigh"],
  ["outseam", "aboveKnee"],
];

describe("anchorToPlatePoint", () => {
  // Repere de controle : le sommet du crane de la figure est a y = 21 sur 700,
  // soit 3 % depuis le haut. L'ancre `height` du catalogue vaut 97 : elle se
  // compte donc depuis les pieds.
  it("place l'ancre du crane en haut de la planche", () => {
    expect(anchorToPlatePoint({ x: 50, y: 97 }).cy).toBeCloseTo(21, 1);
  });

  it("place une ancre au sol tout en bas", () => {
    expect(anchorToPlatePoint({ x: 50, y: 0 }).cy).toBe(PLATE_HEIGHT);
  });

  it("centre horizontalement une ancre a 50 %", () => {
    expect(anchorToPlatePoint({ x: 50, y: 50 }).cx).toBe(PLATE_WIDTH / 2);
  });

  it("respecte l'ordre anatomique de haut en bas", () => {
    const at = (slug: string) => {
      const m = (
        catalog.measurements as { slug: string; anchor: { x: number; y: number } | null }[]
      ).find((x) => x.slug === slug);
      if (!m?.anchor) throw new Error(`ancre absente pour ${slug}`);
      return anchorToPlatePoint(m.anchor).cy;
    };
    const ordre = ["height", "neck", "chest", "waist", "hips", "thigh_left"];
    const y = ordre.map(at);
    for (let i = 1; i < y.length; i++) {
      expect(y[i]).toBeGreaterThan(defined(y[i - 1]));
    }
  });

  // Tolerance de 2 unites sur 700, soit 0,3 % de la figure : le catalogue
  // stocke ses `y` en pourcentage a une decimale, ce qui laisse deja 0,7 unite
  // de jeu par pas. Assez serre pour attraper un repere deplace, assez lache
  // pour ne pas se plaindre d'un arrondi.
  it.each(SUR_UN_REPERE)("place l'ancre %s sur le repere %s du trace", (slug, repere) => {
    const m = (
      catalog.measurements as { slug: string; anchor: { x: number; y: number } | null }[]
    ).find((x) => x.slug === slug);
    if (!m?.anchor) throw new Error(`ancre absente pour ${slug}`);
    const ecart = Math.abs(anchorToPlatePoint(m.anchor).cy - PLATE_LANDMARKS[repere]);
    expect(ecart).toBeLessThan(2);
  });

  it("range les reperes de haut en bas, du sommet du crane a la plante", () => {
    const names = Object.keys(PLATE_LANDMARKS);
    const ys = Object.values(PLATE_LANDMARKS);
    expect(names[0]).toBe("crown");
    expect(names[names.length - 1]).toBe("sole");
    for (let i = 1; i < ys.length; i++) {
      expect(ys[i]).toBeGreaterThan(defined(ys[i - 1]));
    }
    expect(defined(ys[ys.length - 1])).toBeLessThanOrEqual(PLATE_HEIGHT);
  });

  it("garde toutes les ancres du catalogue dans la planche", () => {
    const ancres = (catalog.measurements as { anchor: { x: number; y: number } | null }[])
      .map((m) => m.anchor)
      .filter((a): a is { x: number; y: number } => a != null);
    expect(ancres.length).toBeGreaterThan(0);
    for (const a of ancres) {
      const p = anchorToPlatePoint(a);
      expect(p.cx).toBeGreaterThanOrEqual(0);
      expect(p.cx).toBeLessThanOrEqual(PLATE_WIDTH);
      expect(p.cy).toBeGreaterThanOrEqual(0);
      expect(p.cy).toBeLessThanOrEqual(PLATE_HEIGHT);
    }
  });
});
