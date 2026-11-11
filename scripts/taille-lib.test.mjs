import { describe, expect, it } from "vitest";
import { productionLines, sizeProblems } from "./taille-lib.mjs";

describe("productionLines", () => {
  it("compte un fichier TypeScript en entier", () => {
    expect(productionLines("src/a.ts", "a\nb\nc")).toBe(3);
  });

  it("arrête un fichier Rust au premier #[cfg(test)]", () => {
    expect(productionLines("src/a.rs", "fn a() {}\nfn b() {}\n#[cfg(test)]\nmod tests {}\n")).toBe(
      2
    );
  });
});

describe("sizeProblems", () => {
  it("signale un fichier au-delà du seuil, pas une exception motivée", () => {
    const sizes = [
      { path: "src/gros.ts", lines: 301 },
      { path: "src/garde.rs", lines: 420 },
      { path: "src/petit.ts", lines: 300 },
    ];
    expect(sizeProblems(sizes, 300, { "src/garde.rs": "table de donnees generee" })).toEqual([
      "src/gros.ts : 301 lignes, au-dela de 300",
    ]);
  });

  it("refuse une exception périmée", () => {
    expect(
      sizeProblems([{ path: "src/a.ts", lines: 120 }], 300, {
        "src/a.ts": "ancien motif",
        "src/disparu.ts": "x",
      })
    ).toEqual([
      "src/a.ts : exception perimee (120 lignes)",
      "src/disparu.ts : exception perimee (fichier absent)",
    ]);
  });
});
