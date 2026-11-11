import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { anchorsOf, brokenLinks, extractLinks, resolveTarget } from "./liens-doc-lib.mjs";

const IGNORED_DIRS = new Set([
  "node_modules",
  "target",
  "dist",
  "logs",
  "audits",
  ".git",
  "_backup",
  "worktrees",
]);
const SKIPPED = ["audits", "logs"];

function markdownFiles(dir = ".", out = []) {
  for (const name of readdirSync(dir)) {
    const path = dir === "." ? name : `${dir}/${name}`;
    if (statSync(path).isDirectory()) {
      if (!IGNORED_DIRS.has(name)) markdownFiles(path, out);
    } else if (name.endsWith(".md")) {
      out.push(path);
    }
  }
  return out;
}

describe("extractLinks", () => {
  it("lit les liens, ignore le code et les adresses externes", () => {
    const text =
      '[a](docs/a.md) et `[b](faux.md)`\n```\n[c](faux.md)\n```\n[d](https://x.test) <a href="y.md">';
    expect(extractLinks(text).map((l) => l.target)).toEqual([
      "docs/a.md",
      "https://x.test",
      "y.md",
    ]);
  });
});

describe("brokenLinks", () => {
  const files = {
    "docs/a.md":
      "# Titre\n[ok](b.md#un-titre) [mort](c.md) [ancre](b.md#absente) [haut](../README.md)",
    "docs/b.md": "## Un titre\n",
    "README.md": "x",
  };
  const exists = (p) => p in files;
  const read = (p) => files[p];

  it("voit un lien vers un fichier absent et une ancre absente (cas connu mauvais)", () => {
    expect(brokenLinks(["docs/a.md"], exists, read)).toEqual([
      "docs/a.md:2 : c.md (fichier absent : docs/c.md)",
      "docs/a.md:2 : b.md#absente (ancre absente dans docs/b.md)",
    ]);
  });

  it("laisse passer un lien correct (cas connu bon)", () => {
    expect(brokenLinks(["docs/b.md"], exists, read)).toEqual([]);
    expect(resolveTarget("docs/a.md", "../README.md").path).toBe("README.md");
    expect(anchorsOf("## Un titre\n").has("un-titre")).toBe(true);
  });
});

describe("la documentation du dépôt", () => {
  it("n'a aucun lien relatif mort", () => {
    const files = markdownFiles();
    expect(files.length).toBeGreaterThan(15);
    const exists = (p) => existsSync(join(".", p));
    const read = (p) => readFileSync(join(".", p), "utf-8");
    expect(brokenLinks(files, exists, read, SKIPPED)).toEqual([]);
  });
});
