import { describe, expect, it } from "vitest";
import { describeInsightReport } from "./report";

describe("describeInsightReport", () => {
  it("annonce le nombre d'insights et nomme les analyses en échec", () => {
    expect(describeInsightReport({ generated: 2, failed: ["fréquence cardiaque"] })).toEqual({
      success: "2 insights générés",
      failure: "Analyses en échec : fréquence cardiaque",
    });
  });

  it("ne présente pas un résultat vide comme une réussite chiffrée", () => {
    expect(describeInsightReport({ generated: 0, failed: [] })).toEqual({
      success: "Aucun nouvel insight",
      failure: null,
    });
  });

  it("accorde au singulier", () => {
    expect(describeInsightReport({ generated: 1, failed: [] }).success).toBe("1 insight généré");
  });
});
