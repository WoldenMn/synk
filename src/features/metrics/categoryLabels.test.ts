import { describe, expect, it } from "vitest";
import { METRIC_CATEGORIES, METRIC_CATEGORY_LABELS } from "./categoryLabels";

describe("libellés des catégories de métriques", () => {
  it("nomme les constantes de toutes les sources, pas le seul Body Scan de Withings", () => {
    expect(METRIC_CATEGORY_LABELS.body_scan).toBe("Constantes de santé");
    expect(Object.values(METRIC_CATEGORY_LABELS)).not.toContain("Body Scan");
  });

  it("garde l'ordre d'affichage de l'explorateur", () => {
    expect(METRIC_CATEGORIES).toEqual([
      "body_composition",
      "circumference",
      "body_scan",
      "derived",
      "custom",
    ]);
  });
});
