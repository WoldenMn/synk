import { describe, it, expect } from "vitest";
import { rangeChartLabel, countChartLabel } from "./a11y";

// Normalise les separateurs de milliers fr-FR (espace insecable U+202F / U+00A0)
// en espace simple pour des assertions stables selon l'environnement ICU.
function norm(s: string): string {
  return s
    .split(String.fromCharCode(0x202f))
    .join(" ")
    .split(String.fromCharCode(0x00a0))
    .join(" ");
}

describe("rangeChartLabel", () => {
  it("resume une serie multi-points avec min/max et unité", () => {
    const label = rangeChartLabel({
      title: "Courbe du poids",
      values: [72, 70, 75],
      unit: "kg",
    });
    expect(norm(label)).toBe("Courbe du poids, 3 points, de 70 a 75 kg");
  });

  it("singularise le compteur a 1 point et annonce la valeur", () => {
    const label = rangeChartLabel({ title: "Courbe du poids", values: [80], unit: "kg" });
    expect(norm(label)).toBe("Courbe du poids, 1 point, valeur 80 kg");
  });

  it("retourne un label vide quand la serie est vide", () => {
    expect(rangeChartLabel({ title: "Courbe", values: [], unit: "kg" })).toBe(
      "Graphique vide, aucune donnée"
    );
  });

  it("supporte un pointNoun personnalise et son pluriel", () => {
    const label = rangeChartLabel({
      title: "Histogramme du nombre de pas quotidiens",
      values: [5000, 12000],
      unit: "pas",
      pointNoun: "jour",
    });
    expect(norm(label)).toBe(
      "Histogramme du nombre de pas quotidiens, 2 jours, de 5 000 a 12 000 pas"
    );
  });

  it("omet l'unité quand elle est absente", () => {
    const label = rangeChartLabel({ title: "Score", values: [60, 90] });
    expect(norm(label)).toBe("Score, 2 points, de 60 a 90");
  });

  it("arrondit les valeurs decimales a une decimale", () => {
    const label = rangeChartLabel({ title: "IMC", values: [22.456, 24.123], unit: "" });
    expect(norm(label)).toContain("de 22,5 a 24,1");
  });
});

describe("countChartLabel", () => {
  it("resume un graphique multi-series par son nombre d'éléments", () => {
    const label = countChartLabel({
      title: "Tension artérielle systolique et diastolique",
      count: 8,
      pointNoun: "mesure",
    });
    expect(label).toBe("Tension artérielle systolique et diastolique, 8 mesures");
  });

  it("singularise a 1 élément", () => {
    expect(countChartLabel({ title: "Phases de sommeil", count: 1, pointNoun: "nuit" })).toBe(
      "Phases de sommeil, 1 nuit"
    );
  });

  it("retourne un label vide quand count vaut 0", () => {
    expect(countChartLabel({ title: "Sommeil", count: 0, pointNoun: "nuit" })).toBe(
      "Graphique vide, aucune donnée"
    );
  });
});
