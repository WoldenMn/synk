import { describe, expect, it } from "vitest";
import css from "../styles/globals.css?raw";
import { METRIC_COLORS } from "./constants";
import {
  blendOver,
  contrastRatio,
  contrastRgb,
  parseHsl,
  toRgb,
  parseTokenBlocks,
  resolveTokens,
} from "./contrast";

const blocks = parseTokenBlocks(css);
const THEMES = ["indigo", "emerald", "rose", "amber", "slate"];
const MODES = ["light", "dark"] as const;

const TEXT_PAIRS: [string, string][] = [
  ["foreground", "background"],
  ["card-foreground", "card"],
  ["muted-foreground", "background"],
  ["muted-foreground", "card"],
  ["muted-foreground", "muted"],
  ["destructive-text", "background"],
  ["destructive-text", "card"],
  ["destructive-text", "muted"],
  ["destructive-foreground", "destructive"],
  ["primary-foreground", "primary"],
  ["primary-text", "background"],
  ["primary-text", "card"],
  ["primary-text", "muted"],
  ["success-text", "background"],
  ["success-text", "card"],
  ["success-text", "muted"],
  ["warning-text", "background"],
  ["warning-text", "card"],
  ["warning-text", "muted"],
];

const SERIES_TOKENS = Object.keys(blocks[":root"] ?? {}).filter((k) => k.startsWith("series-"));

const UI_PAIRS: [string, string][] = [
  ...SERIES_TOKENS.flatMap((token): [string, string][] => [
    [token, "background"],
    [token, "card"],
  ]),
  ["border", "background"],
  ["ring", "background"],
  ["success", "background"],
  ["success", "card"],
  ["warning", "background"],
  ["warning", "card"],
];

function ratio(tokens: Record<string, string>, fg: string, bg: string): number {
  const a = parseHsl(tokens[fg] ?? "");
  const b = parseHsl(tokens[bg] ?? "");
  if (!a || !b) throw new Error(`jeton absent ou illisible : --${a ? bg : fg}`);
  return contrastRatio(a, b);
}

describe("instrument de contraste", () => {
  it("rend 21 pour noir sur blanc et 1 pour une couleur sur elle-même", () => {
    expect(contrastRatio([0, 0, 0], [0, 0, 100])).toBeCloseTo(21, 1);
    expect(contrastRatio([243, 75, 59], [243, 75, 59])).toBeCloseTo(1, 5);
  });

  it("retrouve la mesure de l'audit sur le rouge sombre d'avant correctif", () => {
    expect(contrastRatio([0, 63, 31], [224, 50, 6])).toBeCloseTo(1.98, 1);
  });

  it("lit bien les blocs de globals.css", () => {
    expect(Object.keys(blocks)).toEqual(expect.arrayContaining([":root", ".dark", ".theme-rose"]));
  });
});

describe.each(MODES)("contraste des jetons, mode %s", (mode) => {
  it.each(THEMES)("thème %s : textes à 4,5:1, contours à 3:1", (theme) => {
    const tokens = resolveTokens(blocks, mode, theme);
    const failures = [
      ...TEXT_PAIRS.filter(([fg, bg]) => ratio(tokens, fg, bg) < 4.5).map(
        ([fg, bg]) => `${fg}/${bg} ${ratio(tokens, fg, bg).toFixed(2)}`
      ),
      ...UI_PAIRS.filter(([fg, bg]) => ratio(tokens, fg, bg) < 3).map(
        ([fg, bg]) => `${fg}/${bg} ${ratio(tokens, fg, bg).toFixed(2)}`
      ),
    ];
    expect(failures).toEqual([]);
  });
});

const TINT_BASES = ["background", "card", "muted"];
const TINT_ALPHAS = [0.05, 0.1, 0.15];
const TINTED_TEXTS: [string, string][] = [
  ["success-text", "success"],
  ["warning-text", "warning"],
  ["destructive-text", "destructive"],
  ["primary-text", "primary"],
];

function tintedRatio(
  tokens: Record<string, string>,
  text: string,
  tint: string,
  base: string,
  alpha: number
): number {
  const textColor = parseHsl(tokens[text] ?? "");
  const tintColor = parseHsl(tokens[tint] ?? "");
  const baseColor = parseHsl(tokens[base] ?? "");
  if (!textColor || !tintColor || !baseColor)
    throw new Error(`jeton illisible : ${text} ${tint} ${base}`);
  return contrastRgb(toRgb(textColor), blendOver(tintColor, baseColor, alpha));
}

describe("texte sur fond teinté par un jeton d'état", () => {
  it("mélange un jeton sur son fond à l'opacité demandée", () => {
    expect(blendOver([0, 0, 0], [0, 0, 100], 0.5)).toEqual([127.5, 127.5, 127.5]);
  });

  describe.each(MODES)("mode %s", (mode) => {
    it.each(THEMES)("thème %s : 4,5:1 sur les teintes à 5, 10 et 15 %", (theme) => {
      const tokens = resolveTokens(blocks, mode, theme);
      const failures = TINTED_TEXTS.flatMap(([text, tint]) =>
        TINT_BASES.flatMap((base) =>
          TINT_ALPHAS.filter((alpha) => tintedRatio(tokens, text, tint, base, alpha) < 4.5).map(
            (alpha) =>
              `${text} sur ${tint}/${alpha * 100} % de ${base} ${tintedRatio(tokens, text, tint, base, alpha).toFixed(2)}`
          )
        )
      );
      expect(failures).toEqual([]);
    });
  });
});

describe("couleurs des séries de graphique", () => {
  it("existent en clair et en sombre", () => {
    expect(SERIES_TOKENS.length).toBeGreaterThan(15);
    expect(Object.keys(blocks[".dark"] ?? {}).filter((k) => k.startsWith("series-"))).toEqual(
      SERIES_TOKENS
    );
  });

  it("sont toutes lues par METRIC_COLORS depuis un jeton existant", () => {
    for (const color of Object.values(METRIC_COLORS)) {
      const name = /var\(--(series-[\w-]+)\)/.exec(color)?.[1];
      expect(SERIES_TOKENS).toContain(name);
    }
  });
});
