import { describe, expect, it } from "vitest";
import { emptyAlertDraft, parseAlertDraft } from "./alertDraft";

describe("parseAlertDraft", () => {
  it("part d'un seuil vide et refuse de l'enregistrer à 0", () => {
    expect(emptyAlertDraft().threshold).toBe("");
    expect(parseAlertDraft({ ...emptyAlertDraft(), label: "Poids haut" })).toEqual({
      ok: false,
      errors: { threshold: "Indiquez un seuil." },
    });
  });

  it("accepte un seuil à 0 saisi exprès", () => {
    const r = parseAlertDraft({ ...emptyAlertDraft(), threshold: "0", label: "Pas nuls" });
    expect(r).toEqual({
      ok: true,
      input: { metric_key: "weight", condition: "above", threshold: 0, label: "Pas nuls" },
    });
  });

  it("demande un nom", () => {
    const r = parseAlertDraft({ ...emptyAlertDraft(), threshold: "80" });
    expect(r).toEqual({ ok: false, errors: { label: "Donnez un nom à l'alerte." } });
  });
});
