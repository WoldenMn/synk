import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { productionLines, sizeProblems } from "./taille-lib.mjs";

const MAX = 300;

function walk(dir, keep, out = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name).replaceAll("\\", "/");
    if (entry.isDirectory()) walk(path, keep, out);
    else if (keep(path)) out.push(path);
  }
  return out;
}

const isFront = (p) =>
  /\.(ts|tsx)$/.test(p) &&
  !/\.test\.(ts|tsx)$/.test(p) &&
  !p.endsWith(".d.ts") &&
  !p.includes("/__tests__/") &&
  !p.endsWith("test-setup.ts");
const files = [...walk("src", isFront), ...walk("src-tauri/src", (p) => p.endsWith(".rs"))];
const sizes = files.map((path) => ({
  path,
  lines: productionLines(path, readFileSync(path, "utf-8")),
}));
const exceptions = JSON.parse(readFileSync("scripts/taille-exceptions.json", "utf-8"));
const problems = sizeProblems(sizes, MAX, exceptions);

const biggest = [...sizes].sort((a, b) => b.lines - a.lines).slice(0, 3);
console.log(
  `[taille] ${files.length} fichiers ; les plus longs : ${biggest.map((s) => `${s.path} ${s.lines}`).join(", ")}`
);
for (const p of problems) console.error(`[taille] ${p}`);
process.exit(problems.length === 0 ? 0 : 1);
