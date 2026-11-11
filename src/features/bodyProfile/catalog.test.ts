import { describe, it, expect } from "vitest";
import { CATALOG, KEY_MEASUREMENTS, byZone, findMeasurement, ZONES } from "./catalog";

describe("catalogue bodyProfile", () => {
  it("charge les 47 mensurations", () => {
    expect(CATALOG.length).toBe(47);
  });

  it("expose 22 mensurations clés", () => {
    expect(KEY_MEASUREMENTS.length).toBe(22);
  });

  it("retrouve une mensuration par slug", () => {
    expect(findMeasurement("waist")?.label).toBe("Tour de taille");
    expect(findMeasurement("inexistant")).toBeUndefined();
  });

  it("groupe par zone dans l'ordre déclaré", () => {
    const groups = byZone();
    expect(groups.map((g) => g.zone)).toEqual(ZONES.map((z) => z.id));
    const total = groups.reduce((n, g) => n + g.measurements.length, 0);
    expect(total).toBe(47);
  });

  it("trie chaque zone par ordre croissant", () => {
    for (const group of byZone()) {
      const orders = group.measurements.map((m) => m.order);
      expect(orders).toEqual([...orders].sort((a, b) => a - b));
    }
  });
});
