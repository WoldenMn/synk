import { describe, expect, it } from "vitest";
import css from "../styles/globals.css?raw";

const sources = import.meta.glob("/src/**/*.tsx", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const SMOOTH_CLASS = /(?<![\w:-])scroll-smooth(?![\w-])/;
const SMOOTH_RULE = /scroll-behavior\s*:\s*smooth/;

describe("mouvement", () => {
  it("ne fait defiler aucun conteneur en douceur sans condition (prefers-reduced-motion)", () => {
    const offenders = Object.entries(sources)
      .filter(([path]) => !path.endsWith(".test.tsx"))
      .filter(([, text]) => SMOOTH_CLASS.test(text))
      .map(([path]) => path);
    expect(offenders).toEqual([]);
    expect(SMOOTH_RULE.test(css)).toBe(false);
  });

  it("le motif voit un defilement doux nu et laisse passer motion-safe", () => {
    expect(SMOOTH_CLASS.test('className="flex-1 overflow-y-auto scroll-smooth"')).toBe(true);
    expect(SMOOTH_CLASS.test('className="motion-safe:scroll-smooth"')).toBe(false);
    expect(SMOOTH_RULE.test("html { scroll-behavior: smooth; }")).toBe(true);
    expect(Object.keys(sources).length).toBeGreaterThan(100);
  });
});
