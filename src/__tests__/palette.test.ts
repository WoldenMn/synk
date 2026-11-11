import { describe, expect, it } from "vitest";

const sources = import.meta.glob("/src/**/*.tsx", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const NAMED_COLORS =
  "red|rose|pink|fuchsia|purple|violet|indigo|blue|sky|cyan|teal|emerald|green|lime|yellow|amber|orange|gray|slate|zinc|neutral|stone";
const PALETTE = new RegExp(
  `\\b(?:text|bg|border|fill|stroke|from|to|via|ring|divide|outline|decoration|accent|caret|shadow)-(?:${NAMED_COLORS})-\\d{2,3}\\b`,
  "g"
);

const TEXT_WITH_OPACITY =
  /(?<![\w-])(?:[\w-]+:)*text-(?:foreground|card-foreground|muted-foreground|primary-text|success-text|warning-text|destructive-text|destructive-foreground|primary-foreground)\/\d+/g;

const production = Object.entries(sources).filter(([path]) => !path.endsWith(".test.tsx"));

const ARBITRARY_COLOR =
  /(?<![\w-])(?:[\w-]+:)*(?:text|bg|border|fill|stroke|from|to|via|ring|divide|outline|decoration|accent|caret|shadow)-\[(?:#|rgb|hsl|oklch)[^\]]*\]/g;

describe("palette", () => {
  it("n'ecrit aucune couleur arbitraire (hexadecimale ou fonctionnelle) dans une classe", () => {
    const offenders = production.flatMap(([path, text]) =>
      (text.match(ARBITRARY_COLOR) ?? []).map((m) => `${path} : ${m}`)
    );
    expect(offenders).toEqual([]);
  });

  it("le motif de couleur arbitraire voit un hexadecimal et laisse passer un jeton ou une longueur", () => {
    expect("bg-[#6366f1] hover:text-[rgb(1,2,3)]".match(ARBITRARY_COLOR)).toEqual([
      "bg-[#6366f1]",
      "hover:text-[rgb(1,2,3)]",
    ]);
    expect(
      "bg-primary w-[120px] text-[12px] bg-(--series-weight)".match(ARBITRARY_COLOR)
    ).toBeNull();
  });

  it("n'applique aucune opacite a la couleur d'un texte", () => {
    const offenders = production.flatMap(([path, text]) =>
      (text.match(TEXT_WITH_OPACITY) ?? []).map((m) => `${path} : ${m}`)
    );
    expect(offenders).toEqual([]);
  });

  it("le motif d'opacite voit un texte attenue et laisse passer un fond teinte", () => {
    expect("text-muted-foreground/80 hover:text-foreground/60".match(TEXT_WITH_OPACITY)).toEqual([
      "text-muted-foreground/80",
      "hover:text-foreground/60",
    ]);
    expect(
      "bg-muted-foreground/10 border-warning/40 text-muted-foreground".match(TEXT_WITH_OPACITY)
    ).toBeNull();
  });

  it("n'emploie aucune couleur nommee de Tailwind hors des jetons", () => {
    const offenders = production.flatMap(([path, text]) =>
      (text.match(PALETTE) ?? []).map((m) => `${path} : ${m}`)
    );
    expect(offenders).toEqual([]);
  });

  it("n'ecrit pas le texte primaire avec la couleur de remplissage", () => {
    const offenders = production
      .filter(([, text]) => /(?<![\w-])text-primary(?![\w-])/.test(text))
      .map(([path]) => path);
    expect(offenders).toEqual([]);
  });

  it("ne peint aucun survol en couleur d'accent pleine hors des composants de base", () => {
    const offenders = production
      .filter(([path]) => !path.includes("/components/ui/"))
      .filter(([, text]) => /hover:bg-accent(?![\w/-])/.test(text))
      .map(([path]) => path);
    expect(offenders).toEqual([]);
  });

  it("le motif voit une couleur en dur et laisse passer un jeton", () => {
    expect("bg-indigo-600 text-emerald-500/10".match(PALETTE)).toEqual([
      "bg-indigo-600",
      "text-emerald-500",
    ]);
    expect(
      "text-success-text bg-warning/15 border-destructive text-primary-text".match(PALETTE)
    ).toBeNull();
  });
});
