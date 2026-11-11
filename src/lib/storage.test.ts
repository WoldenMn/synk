import { describe, it, expect, beforeEach } from "vitest";
import {
  safeGetItem,
  safeSetItem,
  safeRemoveItem,
  isIsoDate,
  isString,
  isBoolean,
} from "./storage";

describe("safeGetItem", () => {
  beforeEach(() => localStorage.clear());

  it("retourne null si la cle n'existe pas", () => {
    expect(safeGetItem("missing", isString)).toBeNull();
  });

  it("parse JSON et retourne valeur si validateur OK", () => {
    localStorage.setItem("k", JSON.stringify({ done: true }));
    const validator = (v: unknown): v is { done: boolean } =>
      typeof v === "object" && v !== null && "done" in v;
    expect(safeGetItem("k", validator)).toEqual({ done: true });
  });

  it("fallback raw string si JSON parse échoué", () => {
    localStorage.setItem("k", "not-json");
    expect(safeGetItem("k", isString)).toBe("not-json");
  });

  it("supprime la cle et retourne null si validateur rejette", () => {
    localStorage.setItem("k", JSON.stringify(123));
    expect(safeGetItem("k", isString)).toBeNull();
    expect(localStorage.getItem("k")).toBeNull();
  });

  it("retourne null si localStorage throw (quota / private mode)", () => {
    const original = Storage.prototype.getItem;
    Storage.prototype.getItem = () => {
      throw new Error("quota exceeded");
    };
    expect(safeGetItem("k", isString)).toBeNull();
    Storage.prototype.getItem = original;
  });
});

describe("safeSetItem", () => {
  beforeEach(() => localStorage.clear());

  it("stocke string sans JSON serialization", () => {
    expect(safeSetItem("k", "hello")).toBe(true);
    expect(localStorage.getItem("k")).toBe("hello");
  });

  it("serialise objet en JSON", () => {
    expect(safeSetItem("k", { a: 1 })).toBe(true);
    expect(localStorage.getItem("k")).toBe('{"a":1}');
  });

  it("retourne false si localStorage throw", () => {
    const original = Storage.prototype.setItem;
    Storage.prototype.setItem = () => {
      throw new Error("quota exceeded");
    };
    expect(safeSetItem("k", "v")).toBe(false);
    Storage.prototype.setItem = original;
  });
});

describe("safeRemoveItem", () => {
  it("supprime sans throw", () => {
    localStorage.setItem("k", "v");
    expect(() => safeRemoveItem("k")).not.toThrow();
    expect(localStorage.getItem("k")).toBeNull();
  });

  it("absorbe exception", () => {
    const original = Storage.prototype.removeItem;
    Storage.prototype.removeItem = () => {
      throw new Error("oops");
    };
    expect(() => safeRemoveItem("k")).not.toThrow();
    Storage.prototype.removeItem = original;
  });
});

describe("validateurs", () => {
  it("isIsoDate accepte ISO 8601 valide", () => {
    expect(isIsoDate("2026-04-26T10:00:00Z")).toBe(true);
    expect(isIsoDate("2026-04-26")).toBe(true);
    expect(isIsoDate("not-a-date")).toBe(false);
    expect(isIsoDate(123)).toBe(false);
    expect(isIsoDate(null)).toBe(false);
  });

  it("isString discrimine string vs autres types", () => {
    expect(isString("hello")).toBe(true);
    expect(isString("")).toBe(true);
    expect(isString(123)).toBe(false);
    expect(isString(null)).toBe(false);
    expect(isString(undefined)).toBe(false);
  });

  it("isBoolean accepte uniquement true/false", () => {
    expect(isBoolean(true)).toBe(true);
    expect(isBoolean(false)).toBe(true);
    expect(isBoolean("true")).toBe(false);
    expect(isBoolean(0)).toBe(false);
    expect(isBoolean(null)).toBe(false);
  });
});
