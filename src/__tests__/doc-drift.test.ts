import { describe, expect, it } from "vitest";
import libRs from "../../src-tauri/src/lib.rs?raw";
import bootstrapRs from "../../src-tauri/src/db/bootstrap.rs?raw";
import appTsx from "../App.tsx?raw";
import pkgRaw from "../../package.json?raw";
import catalog from "../../docs/commandes.md?raw";
import couverture from "../../docs/couverture.md?raw";
import floorsRaw from "../../scripts/verify-floors.json?raw";
import { registeredCommands, renderCatalog } from "../lib/commandsCatalog";

const rustSources = import.meta.glob("/src-tauri/src/**/*.rs", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;
const docs = import.meta.glob(["/README.md", "/CLAUDE.md", "/docs/*.md"], {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;
const PROSE = Object.entries(docs).filter(
  ([p]) => !p.endsWith("historique-developpement.md") && !p.endsWith("commandes.md")
);

const counts = {
  commandes: registeredCommands(libRs).length,
  pages: new Set([...appTsx.matchAll(/<Route\s+path="(\/[^"]*)"/g)].map((m) => m[1])).size,
  migrations: [...bootstrapRs.matchAll(/migration!\(\s*\d+/g)].length + 1,
};
const version = (JSON.parse(pkgRaw) as { version: string }).version;

function claims(pattern: RegExp): { where: string; value: string }[] {
  return PROSE.flatMap(([path, text]) =>
    text
      .split("\n")
      .flatMap((line, i) =>
        [...line.matchAll(pattern)].map((m) => ({ where: `${path}:${i + 1}`, value: m[1] ?? "" }))
      )
  );
}

describe("documentation à jour", () => {
  it.each([
    ["commandes", /\b(\d+) commandes\b/g],
    ["pages", /\b(\d+) pages\b/g],
    ["migrations", /\b(\d+) migrations\b/g],
  ] as const)("cite le vrai nombre de %s", (what, pattern) => {
    const wrong = claims(pattern).filter((c) => Number(c.value) !== counts[what]);
    expect(wrong, `${what} = ${counts[what]}`).toEqual([]);
  });

  it("lit les trois comptes du code (contrôle de l'instrument)", () => {
    expect(counts.commandes).toBeGreaterThan(100);
    expect(counts.pages).toBeGreaterThan(15);
    expect(counts.migrations).toBeGreaterThan(20);
    expect(claims(/\b(\d+) commandes\b/g).length).toBeGreaterThan(0);
  });

  it("cite l'installeur de la version courante", () => {
    const wrong = claims(/Synk_(\d+\.\d+\.\d+)_x64-setup\.exe/g).filter((c) => c.value !== version);
    expect(wrong, `version ${version}`).toEqual([]);
  });

  it("ne cite pas de compte de tests (il vit dans scripts/verify-floors.json)", () => {
    expect(claims(/\b(\d{2,5}) tests\b/g)).toEqual([]);
  });

  it("garde CLAUDE.md sous 28 Ko, sans exiger sa presence (l'arbre public ne le porte pas)", () => {
    const size = new TextEncoder().encode(docs["/CLAUDE.md"] ?? "").length;
    expect(size).toBeLessThanOrEqual(28 * 1024);
  });

  it("a un catalogue des commandes à jour", () => {
    expect(catalog).toBe(renderCatalog(libRs, rustSources));
  });

  it("couverture.md ne cite aucun fichier Rust disparu", () => {
    const paths = Object.keys(rustSources);
    const cited = [...couverture.matchAll(/`([\w/]+\.rs)`/g)].map((m) => m[1] ?? "");
    expect(cited.length).toBeGreaterThan(3);
    const gone = cited.filter((name) => !paths.some((p) => p.endsWith(`/${name}`)));
    expect(gone).toEqual([]);
  });

  it("couverture.md donne des mesures au moins égales aux planchers", () => {
    const floors = JSON.parse(floorsRaw) as Record<string, number>;
    const measured = (label: string) => {
      const m = couverture.match(new RegExp(`\\|\\s*${label}\\s*\\|\\s*([\\d,]+) %`));
      return Number((m?.[1] ?? "NaN").replace(",", "."));
    };
    expect(measured("TypeScript")).toBeGreaterThanOrEqual(floors.couverture_ts ?? Infinity);
    expect(measured("Rust")).toBeGreaterThanOrEqual(floors.couverture_rust ?? Infinity);
    expect(measured("Rust, `withings/client`")).toBeGreaterThanOrEqual(
      floors.couverture_withings_client ?? Infinity
    );
  });
});
