import { describe, expect, it } from "vitest";
import {
  checkBudget,
  criticalAssets,
  firstScreenAssets,
  gzipKb,
  staticImports,
} from "./bundle-budget-lib.mjs";

const HTML = `<!doctype html><html><head>
<script type="module" crossorigin src="/assets/index-abc.js"></script>
<link rel="modulepreload" crossorigin href="/assets/vendor-def.js">
<link rel="stylesheet" crossorigin href="/assets/index-ghi.css">
</head><body><div id="root"></div></body></html>`;

describe("criticalAssets", () => {
  it("trouve l'entrée et les préchargements, pas la feuille de style", () => {
    expect(criticalAssets(HTML)).toEqual({
      entry: ["/assets/index-abc.js"],
      preloads: ["/assets/vendor-def.js"],
    });
  });

  it("accepte les attributs dans un autre ordre", () => {
    const html = `<link href="/assets/a.js" rel="modulepreload"><script src="/assets/i.js" type="module"></script>`;
    expect(criticalAssets(html)).toEqual({ entry: ["/assets/i.js"], preloads: ["/assets/a.js"] });
  });
});

describe("gzipKb", () => {
  it("mesure la taille compressée, pas la taille brute", () => {
    expect(gzipKb(Buffer.from("a".repeat(100_000)))).toBeLessThan(1);
  });
});

describe("checkBudget", () => {
  const budget = { entryGzipKb: 110, criticalGzipKb: 150, firstScreenGzipKb: 250 };

  it("passe sous les plafonds", () => {
    expect(checkBudget({ entryKb: 100, criticalKb: 140, firstScreenKb: 240 }, budget)).toEqual({
      ok: true,
      problems: [],
    });
  });

  it("échoue au-dessus et donne la mesure et le plafond", () => {
    const r = checkBudget({ entryKb: 120, criticalKb: 160, firstScreenKb: 260 }, budget);
    expect(r.ok).toBe(false);
    expect(r.problems).toEqual([
      "chunk d'entree : 120 ko gzip, plafond 110 ko",
      "chemin critique : 160 ko gzip, plafond 150 ko",
      "premier ecran : 260 ko gzip, plafond 250 ko",
    ]);
  });

  it("refuse un plafond de premier écran absent plutôt que de passer à vide", () => {
    const r = checkBudget(
      { entryKb: 1, criticalKb: 1, firstScreenKb: 1 },
      { entryGzipKb: 110, criticalGzipKb: 150 }
    );
    expect(r.ok).toBe(false);
    expect(r.problems).toEqual(["premier ecran : plafond absent de bundle-budget.json"]);
  });
});

describe("staticImports", () => {
  it("rend les importations statiques, pas les dynamiques", () => {
    const js = `import{r as b}from"./index-A.js";import"./side-B.js";const L=()=>import("./Page-C.js");export{x}from'./re-D.js'`;
    expect(staticImports(js)).toEqual(["index-A.js", "side-B.js", "re-D.js"]);
  });

  it("ignore une importation dynamique collée à un from", () => {
    expect(staticImports(`const a=from;import("./Lazy-X.js")`)).toEqual([]);
  });
});

describe("firstScreenAssets", () => {
  const files = {
    "index.js": `import"./vendor.js";const D=()=>import("./Dashboard-abc.js");const S=()=>import("./Sleep-def.js")`,
    "vendor.js": `export const v=1`,
    "Dashboard-abc.js": `import{a}from"./index.js";import{c}from"./charts.js";`,
    "charts.js": `import{r}from"./recharts.js";`,
    "recharts.js": `export const r=1`,
    "Sleep-def.js": `import{c}from"./sleep-only.js";`,
    "sleep-only.js": `export const s=1`,
  };
  const read = (name) => files[name];

  it("suit l'entrée, la page du premier écran et leurs importations statiques", () => {
    expect(firstScreenAssets({ entry: "index.js", page: "Dashboard", read }).sort()).toEqual([
      "Dashboard-abc.js",
      "charts.js",
      "index.js",
      "recharts.js",
      "vendor.js",
    ]);
  });

  it("n'embarque pas une autre page", () => {
    const assets = firstScreenAssets({ entry: "index.js", page: "Dashboard", read });
    expect(assets).not.toContain("Sleep-def.js");
    expect(assets).not.toContain("sleep-only.js");
  });

  it("lit une importation dynamique écrite entre accents graves (Rolldown)", () => {
    const rolldown = {
      ...files,
      "index.js": 'import"./vendor.js";const D=()=>B(()=>import(`./Dashboard-abc.js`),[]);',
    };
    expect(
      firstScreenAssets({ entry: "index.js", page: "Dashboard", read: (n) => rolldown[n] }).sort()
    ).toEqual(["Dashboard-abc.js", "charts.js", "index.js", "recharts.js", "vendor.js"]);
  });

  it("échoue si la page du premier écran est introuvable", () => {
    expect(() => firstScreenAssets({ entry: "index.js", page: "Absent", read })).toThrow(/Absent/);
  });

  it("échoue sur un chunk référencé et absent", () => {
    const broken = (name) => (name === "charts.js" ? undefined : files[name]);
    expect(() => firstScreenAssets({ entry: "index.js", page: "Dashboard", read: broken })).toThrow(
      /charts[.]js/
    );
  });
});
