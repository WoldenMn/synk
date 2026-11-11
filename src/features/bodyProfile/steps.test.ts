import { describe, it, expect } from "vitest";
import { buildSteps } from "./steps";

describe("buildSteps", () => {
  it("produit 33 étapes en parcours complet", () => {
    expect(buildSteps("full").length).toBe(33);
  });

  it("produit 16 étapes en parcours essentiel", () => {
    expect(buildSteps("essential").length).toBe(16);
  });

  it("fusionne les deux côtés d'une paire sur une seule étape", () => {
    const step = buildSteps("full").find((s) => s.id === "bicep");
    expect(step).toBeDefined();
    expect(step!.label).toBe("Tour de bras");
    expect(step!.fields.map((f) => f.slug)).toEqual(["bicep_left", "bicep_right"]);
  });

  it("laisse un seul champ aux mesures non latéralisées", () => {
    const step = buildSteps("full").find((s) => s.id === "waist");
    expect(step!.fields.length).toBe(1);
    expect(step!.fields[0]?.side).toBe("none");
  });

  it("ordonne les étapes par ordre croissant du catalogue", () => {
    const orders = buildSteps("full").map((s) => s.order);
    expect(orders).toEqual([...orders].sort((a, b) => a - b));
  });

  it("porte les bornes et l'aide sur chaque étape", () => {
    const step = buildSteps("full").find((s) => s.id === "wrist")!;
    expect(step.guidance).toContain("poignet");
    expect(step.fields[0]?.min).toBe(10);
    expect(step.fields[0]?.max).toBe(30);
  });

  it("ne garde que des mesures clés en parcours essentiel", () => {
    const slugs = buildSteps("essential").flatMap((s) => s.fields.map((f) => f.slug));
    expect(slugs).toContain("waist");
    expect(slugs).not.toContain("head");
  });
});
