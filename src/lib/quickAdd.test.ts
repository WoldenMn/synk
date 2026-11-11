import { describe, expect, it } from "vitest";
import { showsQuickAdd } from "./quickAdd";

describe("showsQuickAdd", () => {
  it("n'offre l'ajout rapide que là où l'on saisit une mesure", () => {
    for (const path of ["/", "/body", "/blood-pressure", "/measurements"]) {
      expect(showsQuickAdd(path)).toBe(true);
    }
    for (const path of ["/export", "/settings", "/profile", "/sleep", "/labs"]) {
      expect(showsQuickAdd(path)).toBe(false);
    }
  });
});
