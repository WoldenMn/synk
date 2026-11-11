import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { renderCatalog } from "../src/lib/commandsCatalog.ts";

const sources = {};
(function walk(dir) {
  for (const entry of readdirSync(dir)) {
    const p = join(dir, entry);
    if (statSync(p).isDirectory()) walk(p);
    else if (p.endsWith(".rs")) sources[p.replaceAll("\\", "/")] = readFileSync(p, "utf8");
  }
})("src-tauri/src");
const lib = readFileSync("src-tauri/src/lib.rs", "utf8");
writeFileSync("docs/commandes.md", renderCatalog(lib, sources));
console.log("[doc] docs/commandes.md regenere");
