import { describe, expect, it } from "vitest";
import { historyHint } from "./hint";

describe("historyHint", () => {
  it("propose de synchroniser quand une source est connectée", () => {
    expect(historyHint(true, "ou ajoutez une nuit à la main")).toBe(
      "Lancez une synchronisation depuis l'en-tête ou ajoutez une nuit à la main."
    );
  });

  it("propose de connecter une source quand aucune ne l'est", () => {
    expect(historyHint(false, "pour importer vos pas et vos calories")).toBe(
      "Connectez une source dans Paramètres pour importer vos pas et vos calories."
    );
  });

  it("traite un état inconnu comme non connecté", () => {
    expect(historyHint(undefined, "ou ajoutez une nuit à la main")).toBe(
      "Connectez une source dans Paramètres ou ajoutez une nuit à la main."
    );
  });
});
