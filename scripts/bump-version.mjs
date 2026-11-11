import { readFileSync, writeFileSync } from "node:fs";
import { bumpText, disagreements, isSemver, readVersions, VERSION_FILES } from "./version-lib.mjs";

const version = process.argv[2];
if (!version || !isSemver(version)) {
  console.error("usage : npm run bump -- X.Y.Z");
  process.exit(1);
}
for (const path of VERSION_FILES) {
  const original = readFileSync(path, "utf8");
  const crlf = original.includes("\r\n");
  const next = bumpText(path, original.replace(/\r\n/g, "\n"), version);
  writeFileSync(path, crlf ? next.replace(/\n/g, "\r\n") : next);
}
const after = readVersions((p) => readFileSync(p, "utf8"));
const wrong = Object.entries(after).filter(([, v]) => v !== version);
if (wrong.length > 0 || Object.keys(disagreements(after)).length > 0) {
  console.error(`[bump] emplacements non montes : ${JSON.stringify(Object.fromEntries(wrong))}`);
  process.exit(1);
}
console.log(`[bump] ${version} dans ${Object.keys(after).length} emplacements`);
