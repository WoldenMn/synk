import { describe, it, expect } from "vitest";
import { formatMeasure, formatProfileDate, formatRecordValue } from "./format";

// toLocaleString("fr-FR") separe les milliers par une espace fine insecable.
const plain = (s: string) =>
  s.replace(new RegExp(`[${String.fromCharCode(0x202f, 0xa0)}]`, "g"), " ");

describe("formatMeasure", () => {
  it("arrondit a une decimale avec la virgule francaise", () => {
    expect(formatMeasure(21.647000000000002, "%")).toBe("21,6 %");
    expect(formatMeasure(76.037, "kg")).toBe("76,0 kg");
  });

  it("garde les entiers sans decimale pour les unites de comptage", () => {
    expect(plain(formatMeasure(12345, "pas"))).toBe("12 345 pas");
    expect(formatMeasure(43, "bpm")).toBe("43 bpm");
  });

  it("rend un tiret pour une valeur absente", () => {
    expect(formatMeasure(null, "kg")).toBe("—");
    expect(formatMeasure(undefined, "cm")).toBe("—");
  });
});

describe("formatRecordValue", () => {
  it("affiche une duree en secondes en heures et minutes", () => {
    expect(formatRecordValue(41640, "s")).toBe("11h 34min");
  });

  it("colle le score a son bareme", () => {
    expect(formatRecordValue(93, "/100")).toBe("93/100");
  });

  it("formate les autres unites comme une mesure", () => {
    expect(formatRecordValue(70.287, "kg")).toBe("70,3 kg");
  });
});

describe("formatProfileDate", () => {
  it("rend une date ISO lisible", () => {
    expect(formatProfileDate("2026-06-28T06:53:52Z")).toMatch(/^28 juin 2026$/);
    expect(formatProfileDate("2026-04-07")).toBe("7 avr. 2026");
  });

  it("rend un tiret sans date", () => {
    expect(formatProfileDate(null)).toBe("—");
  });
});
