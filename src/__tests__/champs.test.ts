import { describe, expect, it } from "vitest";
import css from "../styles/globals.css?raw";

const sources = import.meta.glob("/src/**/*.tsx", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const RAW_FIELD = /<(?:select|textarea)\b|<input\b(?![^>]*type="(?:checkbox|radio|file|hidden)")/g;

describe("champs", () => {
  it("passent par les composants Input, Select et Textarea", () => {
    const offenders = Object.entries(sources)
      .filter(([path]) => !path.includes("/components/ui/") && !path.endsWith(".test.tsx"))
      .flatMap(([path, text]) => (text.match(RAW_FIELD) ?? []).map((m) => `${path} : ${m}`));
    expect(offenders).toEqual([]);
  });

  it("le motif voit un champ brut et laisse passer un composant et une case", () => {
    expect('<input type="date" />'.match(RAW_FIELD)).toHaveLength(1);
    expect('<select className="a">'.match(RAW_FIELD)).toHaveLength(1);
    expect("<textarea />".match(RAW_FIELD)).toHaveLength(1);
    expect('<Input /> <Select> <Textarea /> <input type="checkbox" />'.match(RAW_FIELD)).toBeNull();
  });
});

describe("controles natifs", () => {
  it("annoncent le mode au navigateur", () => {
    expect(css).toMatch(/:root\s*\{[^}]*color-scheme:\s*light/);
    expect(css).toMatch(/\.dark\s*\{[^}]*color-scheme:\s*dark/);
  });

  it("colorent cases et boutons radio a l'accent", () => {
    expect(css).toMatch(/accent-color:\s*hsl\(var\(--primary\)\)/);
  });

  it("gardent un texte indicatif lisible", () => {
    expect(css).toMatch(/placeholder[^}]*color:\s*hsl\(var\(--muted-foreground\)\)/);
  });
});
