import { readFileSync } from "node:fs";
import { join } from "node:path";
import { checkBudget, criticalAssets, firstScreenAssets, gzipKb } from "./bundle-budget-lib.mjs";

const dist = "dist";
const FIRST_SCREEN_PAGE = "Dashboard";
const html = readFileSync(join(dist, "index.html"), "utf-8");
const { entry, preloads } = criticalAssets(html);
if (entry.length !== 1) {
  console.error(`[budget] point d'entree introuvable ou multiple : ${entry.join(", ") || "aucun"}`);
  process.exit(1);
}
const sizeOf = (asset) => gzipKb(readFileSync(join(dist, asset.replace(/^\//, ""))));
const sumKb = (assets) => Math.round(assets.map(sizeOf).reduce((a, v) => a + v, 0) * 10) / 10;
const entryKb = sizeOf(entry[0]);
const criticalKb = sumKb([entry[0], ...preloads]);

let firstScreen;
try {
  const entryName = entry[0].replace(/^\/assets\//, "");
  firstScreen = firstScreenAssets({
    entry: entryName,
    page: FIRST_SCREEN_PAGE,
    read: (name) => {
      try {
        return readFileSync(join(dist, "assets", name), "utf-8");
      } catch {
        return undefined;
      }
    },
  }).map((name) => `/assets/${name}`);
} catch (e) {
  console.error(`[budget] premier ecran illisible : ${e.message}`);
  process.exit(1);
}
const firstScreenKb = sumKb(firstScreen);
const budget = JSON.parse(readFileSync("scripts/bundle-budget.json", "utf-8"));

console.log(
  `[budget] entree ${entry[0]} : ${entryKb} ko gzip ; chemin critique ${criticalKb} ko gzip ; prechargements : ${preloads.join(", ") || "aucun"}`
);
console.log(
  `[budget] premier ecran (${FIRST_SCREEN_PAGE}) : ${firstScreenKb} ko gzip en ${firstScreen.length} fichiers`
);
const { ok, problems } = checkBudget({ entryKb, criticalKb, firstScreenKb }, budget);
for (const problem of problems) console.error(`[budget] DEPASSE ${problem}`);
process.exit(ok ? 0 : 1);
