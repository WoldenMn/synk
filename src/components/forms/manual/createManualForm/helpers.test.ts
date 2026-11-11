import { describe, expect, it } from "vitest";
import type { FormField } from "../types";
import { validateAll } from "./helpers";

const fields: FormField[] = [
  { name: "weight", label: "Poids", type: "decimal", required: true, min: 1, max: 400 },
  { name: "note", label: "Note", type: "text", required: true },
];

describe("validateAll", () => {
  it("donne un message par champ invalide", () => {
    expect(validateAll(fields, { weight: "", note: "" })).toEqual({
      weight: "Poids est requis",
      note: "Note est requis",
    });
  });

  it("ne dit rien d'un champ valide", () => {
    expect(validateAll(fields, { weight: "72", note: "" })).toEqual({ note: "Note est requis" });
    expect(validateAll(fields, { weight: "72", note: "ok" })).toEqual({});
  });

  it("garde les bornes numériques", () => {
    expect(validateAll(fields, { weight: "500", note: "ok" })).toEqual({
      weight: "Poids doit être <= 400",
    });
  });
});
