import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync } from "node:fs";
import { checkCoverage, combine, summarizeLlvmCov } from "./verify-lib.mjs";

const floors = JSON.parse(readFileSync("scripts/verify-floors.json", "utf-8"));
mkdirSync("logs", { recursive: true });
const out = "../logs/llvm-cov.json";

const run = spawnSync(
  "cargo",
  ["llvm-cov", "-j", "2", "--json", "--summary-only", "--output-path", out],
  { cwd: "src-tauri", env: { ...process.env, CARGO_INCREMENTAL: "0" }, stdio: "inherit" }
);
if (run.status !== 0) {
  console.error(`[couverture rust] cargo llvm-cov en echec (code ${run.status})`);
  process.exit(1);
}
const { total, prefix } = summarizeLlvmCov(
  readFileSync("logs/llvm-cov.json", "utf-8"),
  "src/sources/withings/client/"
);
const verdict = combine(
  checkCoverage("couverture rust", total, floors.couverture_rust),
  checkCoverage("withings/client", prefix, floors.couverture_withings_client)
);
console.log(`[couverture rust] ${verdict.message}`);
process.exit(verdict.ok ? 0 : 1);
