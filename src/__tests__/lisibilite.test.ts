import { describe, expect, it } from "vitest";
import tauriConfRaw from "../../src-tauri/tauri.conf.json?raw";

const sources = import.meta.glob("/src/**/*.tsx", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const scripts = import.meta.glob("/src/**/*.{ts,tsx}", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const SMALL_FONT_SIZE = /fontSize(?:=\{|:\s*)(?:[0-9]|1[01])\b/;

describe("lisibilité", () => {
  it("ne descend jamais sous 12 px dans les graphiques (fontSize)", () => {
    const offenders = Object.entries(scripts)
      .filter(([path]) => !/\.test\.tsx?$/.test(path))
      .filter(([, text]) => SMALL_FONT_SIZE.test(text))
      .map(([path]) => path);
    expect(offenders).toEqual([]);
  });

  it("repère un fontSize trop petit et laisse passer 12 px (contrôle de l'instrument)", () => {
    expect(SMALL_FONT_SIZE.test("{ fontSize: 11, fill: x }")).toBe(true);
    expect(SMALL_FONT_SIZE.test("<Text fontSize={10} />")).toBe(true);
    expect(SMALL_FONT_SIZE.test("{ fontSize: 9 }")).toBe(true);
    expect(SMALL_FONT_SIZE.test("{ fontSize: 12 }")).toBe(false);
    expect(SMALL_FONT_SIZE.test("{ fontSize: 13 }")).toBe(false);
    expect(Object.keys(scripts).some((p) => p.endsWith("/chartTheme.ts"))).toBe(true);
  });

  it("ne descend jamais sous 12 px", () => {
    const offenders = Object.entries(sources)
      .filter(([, text]) => /text-\[(?:[0-9]|1[01])px\]/.test(text))
      .map(([path]) => path);
    expect(offenders).toEqual([]);
  });

  it("repère une classe trop petite (contrôle de l'instrument)", () => {
    expect(/text-\[(?:[0-9]|1[01])px\]/.test('<p className="text-[11px]">')).toBe(true);
    expect(/text-\[(?:[0-9]|1[01])px\]/.test('<p className="text-xs">')).toBe(false);
    expect(Object.keys(sources).length).toBeGreaterThan(100);
  });

  it("laisse zoomer la fenêtre (Ctrl + molette, Ctrl +/-)", () => {
    expect(JSON.parse(tauriConfRaw).app.windows[0].zoomHotkeysEnabled).toBe(true);
  });
});
