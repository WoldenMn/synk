import { describe, expect, it } from "vitest";
import { emptyLabDraft, parseLabDraft } from "./labDraft";

const filled = { ...emptyLabDraft("2026-10-03"), test_name: "Glycémie", value: "0.95" };

describe("parseLabDraft", () => {
  it("part de champs vides, datés du jour local", () => {
    expect(emptyLabDraft("2026-10-03")).toMatchObject({
      date: "2026-10-03",
      value: "",
      test_name: "",
    });
  });

  it("refuse une valeur vide au lieu d'enregistrer 0", () => {
    const r = parseLabDraft({ ...filled, value: "" });
    expect(r).toEqual({ ok: false, errors: { value: "Indiquez la valeur du résultat." } });
  });

  it("nomme chaque champ en faute", () => {
    const r = parseLabDraft({ ...filled, test_name: " ", reference_min: "2", reference_max: "1" });
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(r.errors.test_name).toBe("Indiquez le nom de l'analyse.");
      expect(r.errors.reference_max).toBe("Doit être supérieure ou égale à la référence min.");
    }
  });

  it("rend une entrée typée quand tout est bon", () => {
    expect(parseLabDraft({ ...filled, unit: "g/L", reference_min: "0.7" })).toEqual({
      ok: true,
      input: {
        date: "2026-10-03",
        test_name: "Glycémie",
        value: 0.95,
        unit: "g/L",
        reference_min: 0.7,
        reference_max: null,
        notes: null,
      },
    });
  });
});
