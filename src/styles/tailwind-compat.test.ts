import { describe, expect, it } from "vitest";
import css from "./globals.css?raw";
import { defined } from "@/test-utils/defined";

const sources = import.meta.glob<string>("/src/**/*.{ts,tsx}", {
  query: "?raw",
  import: "default",
  eager: true,
});

function customUtilities(text: string): string[] {
  return [...text.matchAll(/@utility\s+([\w*-]+)\s*\{/g)].map((m) => defined(m[1]));
}

function glowClassesUsed(files: Record<string, string>): string[] {
  const used = new Set<string>();
  for (const [path, text] of Object.entries(files)) {
    if (path.endsWith(".test.ts") || path.endsWith(".test.tsx")) continue;
    for (const m of text.matchAll(/(?<![\w-])glow-[a-z]+/g)) used.add(m[0]);
  }
  return [...used].sort();
}

describe("compatibilité Tailwind 4 de globals.css", () => {
  it("déclare en @utility chaque classe glow-* employée par le code", () => {
    const used = glowClassesUsed(sources);
    expect(used.length).toBeGreaterThan(0);
    const declared = customUtilities(css);
    expect(used.filter((c) => !declared.includes(c))).toEqual([]);
  });

  it("repère une classe glow-* qui n'a plus son @utility (contrôle de l'instrument)", () => {
    const broken = css.replace("@utility glow-rose", "@utility glow-rose-absent");
    const declared = customUtilities(broken);
    expect(glowClassesUsed(sources).filter((c) => !declared.includes(c))).toContain("glow-rose");
  });

  it("garde la variante sombre par classe et les espacements à la formule de la v3", () => {
    expect(css).toContain("@custom-variant dark (&:where(.dark, .dark *))");
    expect(customUtilities(css)).toEqual(expect.arrayContaining(["space-y-*", "space-x-*"]));
  });

  it("ne scanne que le code de l'application et index.html", () => {
    expect(css).toContain('@import "tailwindcss" source(none)');
    expect(css).toContain('@source "../../index.html"');
  });

  it("rend les couleurs du thème sous leurs noms Tailwind, destructive-text compris", () => {
    for (const name of ["background", "primary", "muted-foreground", "destructive-text"]) {
      expect(css).toContain(`--color-${name}: hsl(var(--${name}))`);
    }
  });
});
