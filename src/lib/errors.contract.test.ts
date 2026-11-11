import { describe, it, expect } from "vitest";
import errorRs from "../../src-tauri/src/error.rs?raw";
import { formatError, type RustErrorKind } from "./errors";

const enumStart = errorRs.indexOf("pub enum AppError");
const enumBody = errorRs.slice(enumStart, errorRs.indexOf("\n}", enumStart));
const variants = [...enumBody.matchAll(/^ {4}([A-Z][A-Za-z]+)\s*[({]/gm)].map((m) => m[1]);

const kindStart = errorRs.indexOf("pub fn kind(&self)");
const kindBody = errorRs.slice(kindStart, errorRs.indexOf("\n    }\n", kindStart));
const kinds = [
  ...kindBody.matchAll(/AppError::([A-Za-z]+)(?:\(_\)|\s*\{\s*\.\.\s*\})\s*=>\s*"([A-Za-z]+)"/g),
].map((m) => ({ variant: m[1], kind: m[2] }));

describe("contrat des erreurs Rust vers TypeScript", () => {
  it("lit les variantes d'AppError et le match de kind()", () => {
    expect(variants.length).toBeGreaterThanOrEqual(10);
    expect(kinds.length).toBe(variants.length);
  });

  it("seules les variantes déclarées en alias partagent le kind d'une autre", () => {
    const differing = kinds.filter((k) => k.kind !== k.variant);
    expect(differing).toEqual([{ variant: "NotFoundByKey", kind: "NotFound" }]);
  });

  it("chaque variante a un kind du même nom", () => {
    expect(kinds.map((k) => k.variant).sort()).toEqual([...variants].sort());
    for (const { variant, kind } of kinds.filter((k) => k.variant !== "NotFoundByKey")) {
      expect(kind).toBe(variant);
    }
  });

  it.each(kinds.map((k) => k.kind))("le kind %s a un code et un message utilisateur", (kind) => {
    const e = formatError({ kind: kind as RustErrorKind, message: "m", detail: "d" });
    expect(e.code, kind).not.toBe("UNKNOWN");
    expect(e.userMessage.length, kind).toBeGreaterThan(0);
    expect(e.userMessage, kind).not.toBe("m");
  });
});
