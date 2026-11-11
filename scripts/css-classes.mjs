import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { cssClassNames } from "./css-classes-lib.mjs";

const dir = "dist/assets";
const css = readdirSync(dir)
  .filter((f) => f.endsWith(".css"))
  .map((f) => readFileSync(join(dir, f), "utf8"))
  .join("\n");
if (css.length === 0) {
  console.error("[css-classes] aucun CSS dans dist/assets : lancer npm run build d'abord");
  process.exit(1);
}
console.log(cssClassNames(css).join("\n"));
