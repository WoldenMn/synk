export type Hsl = readonly [number, number, number];
export type TokenBlocks = Record<string, Record<string, string>>;

export function parseHsl(value: string): Hsl | null {
  const m = value.match(/(\d+(?:\.\d+)?)\s+(\d+(?:\.\d+)?)%\s+(\d+(?:\.\d+)?)%/);
  return m ? [Number(m[1]), Number(m[2]), Number(m[3])] : null;
}

function hslToRgb([h, s, l]: Hsl): [number, number, number] {
  const sat = s / 100;
  const lig = l / 100;
  const c = (1 - Math.abs(2 * lig - 1)) * sat;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = lig - c / 2;
  const [r, g, b] =
    h < 60
      ? [c, x, 0]
      : h < 120
        ? [x, c, 0]
        : h < 180
          ? [0, c, x]
          : h < 240
            ? [0, x, c]
            : h < 300
              ? [x, 0, c]
              : [c, 0, x];
  return [(r + m) * 255, (g + m) * 255, (b + m) * 255];
}

function luminance(rgb: [number, number, number]): number {
  const linear = rgb.map((v) => {
    const c = v / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * (linear[0] ?? 0) + 0.7152 * (linear[1] ?? 0) + 0.0722 * (linear[2] ?? 0);
}

export type Rgb = [number, number, number];

export function toRgb(color: Hsl): Rgb {
  return hslToRgb(color);
}

export function blendOver(fg: Hsl, bg: Hsl, alpha: number): Rgb {
  const a = hslToRgb(fg);
  const b = hslToRgb(bg);
  return [
    a[0] * alpha + b[0] * (1 - alpha),
    a[1] * alpha + b[1] * (1 - alpha),
    a[2] * alpha + b[2] * (1 - alpha),
  ];
}

export function contrastRgb(a: Rgb, b: Rgb): number {
  const la = luminance(a);
  const lb = luminance(b);
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}

export function contrastRatio(a: Hsl, b: Hsl): number {
  return contrastRgb(hslToRgb(a), hslToRgb(b));
}

export function parseTokenBlocks(css: string): TokenBlocks {
  const blocks: TokenBlocks = {};
  for (const m of css.matchAll(/(:root|\.dark\.theme-\w+|\.dark|\.theme-\w+)\s*\{([^}]+)\}/g)) {
    const selector = m[1];
    const body = m[2];
    if (selector === undefined || body === undefined) continue;
    const vars = blocks[selector] ?? {};
    for (const v of body.matchAll(/--([\w-]+):\s*([^;]+);/g)) {
      const name = v[1];
      const value = v[2];
      if (name !== undefined && value !== undefined) vars[name] = value.trim();
    }
    blocks[selector] = vars;
  }
  return blocks;
}

export function resolveTokens(
  blocks: TokenBlocks,
  mode: "light" | "dark",
  theme: string
): Record<string, string> {
  return {
    ...blocks[":root"],
    ...(mode === "dark" ? blocks[".dark"] : {}),
    ...blocks[`.theme-${theme}`],
    ...(mode === "dark" ? blocks[`.dark.theme-${theme}`] : {}),
  };
}
