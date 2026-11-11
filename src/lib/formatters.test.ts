import { describe, it, expect } from "vitest";
import {
  formatDate,
  formatDateTime,
  formatNumber,
  formatWeight,
  formatPercent,
  formatDuration,
  formatBP,
  orDash,
} from "./formatters";

describe("formatDate", () => {
  it("formate une date ISO en jour mois annee", () => {
    expect(formatDate("2026-04-23T12:00:00Z")).toMatch(/23 avr\.? 2026/);
  });

  it("retourne un tiret pour une chaine vide", () => {
    expect(formatDate("")).toBe("\u2014");
  });
});

describe("formatDateTime", () => {
  it("inclut l'heure dans le format", () => {
    expect(formatDateTime("2026-04-23T12:30:00Z")).toMatch(/\d{2}:\d{2}/);
  });

  it("retourne un tiret pour une chaine vide", () => {
    expect(formatDateTime("")).toBe("\u2014");
  });
});

describe("formatNumber", () => {
  it("formate avec 1 decimale par defaut", () => {
    expect(formatNumber(42.56)).toMatch(/42[,.]6/);
  });

  it("respecte le paramètre decimals", () => {
    expect(formatNumber(42.56789, 3)).toMatch(/42[,.]568/);
  });

  it("retourne un tiret pour null", () => {
    expect(formatNumber(null as unknown as number)).toBe("\u2014");
  });

  it("retourne un tiret pour NaN", () => {
    expect(formatNumber(NaN)).toBe("\u2014");
  });
});

describe("formatWeight", () => {
  it("ajoute l'unité kg", () => {
    expect(formatWeight(72.5)).toMatch(/kg$/);
  });

  it("retourne un tiret pour null", () => {
    expect(formatWeight(null as unknown as number)).toBe("\u2014");
  });
});

describe("formatPercent", () => {
  it("ajoute le signe pourcent", () => {
    expect(formatPercent(15.3)).toMatch(/%$/);
  });
});

describe("formatDuration", () => {
  it("formate en minutes seulement si moins d'une heure", () => {
    expect(formatDuration(1800)).toBe("30min");
  });

  it("formate en heures et minutes au-dela de 3600s", () => {
    expect(formatDuration(3660)).toBe("1h 01min");
  });

  it("retourne un tiret pour valeur negative", () => {
    expect(formatDuration(-10)).toBe("\u2014");
  });

  it("retourne un tiret pour null", () => {
    expect(formatDuration(null as unknown as number)).toBe("\u2014");
  });
});

describe("formatBP", () => {
  it("formate en sys/dia", () => {
    expect(formatBP(120, 80)).toBe("120/80");
  });

  it("retourne un tiret si sys null", () => {
    expect(formatBP(null as unknown as number, 80)).toBe("\u2014");
  });
});

describe("orDash", () => {
  it("rend un tiret pour null et undefined", () => {
    expect(orDash(null, formatWeight)).toBe("—");
    expect(orDash(undefined, formatWeight)).toBe("—");
  });

  it("formate zero au lieu de le confondre avec une valeur absente", () => {
    expect(orDash(0, (v) => `${v} kg`)).toBe("0 kg");
  });

  it("applique le formateur a une valeur presente", () => {
    expect(orDash(72.4, formatWeight)).toBe(formatWeight(72.4));
  });
});
