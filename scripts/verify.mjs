import { execFileSync, spawnSync } from "node:child_process";
import { appendFileSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import {
  checkCoverage,
  checkFloor,
  combine,
  judgeStep,
  parseCargoTests,
  readCoverageSummary,
  readVitestReport,
} from "./verify-lib.mjs";

const rapide = process.argv.includes("--rapide");
const node = process.execPath;
const rustEnv = { ...process.env, CARGO_INCREMENTAL: "0" };
const vitestJson = "logs/vitest-report.json";
const coverageSummary = "logs/coverage/coverage-summary.json";

let floors = {};
try {
  floors = JSON.parse(readFileSync("scripts/verify-floors.json", "utf-8"));
} catch (e) {
  console.error(`[verify] scripts/verify-floors.json illisible : ${e.message}`);
}

function gitState() {
  const git = (args) => execFileSync("git", args, { encoding: "utf-8" }).trim();
  try {
    return {
      commit: git(["rev-parse", "HEAD"]),
      tree: git(["status", "--porcelain", "--untracked-files=all"])
        ? "modifie"
        : git(["rev-parse", "HEAD^{tree}"]),
    };
  } catch (e) {
    console.error(`[verify] état git illisible : ${e.message}`);
    return { commit: "inconnu", tree: "inconnu" };
  }
}

const before = gitState();

mkdirSync("logs", { recursive: true });
rmSync(vitestJson, { force: true });
rmSync(coverageSummary, { force: true });

const steps = [
  {
    name: "prettier",
    cmd: node,
    args: [
      "node_modules/prettier/bin/prettier.cjs",
      "--check",
      "src",
      "scripts",
      "*.config.{ts,js,mjs}",
      "tsconfig*.json",
      "components.json",
    ],
  },
  { name: "tsc", cmd: node, args: ["node_modules/typescript/bin/tsc", "--noEmit"] },
  {
    name: "eslint",
    cmd: node,
    args: ["node_modules/eslint/bin/eslint.js", "src", "scripts", "--max-warnings", "0"],
  },
  { name: "taille", cmd: node, args: ["scripts/taille-fichiers.mjs"] },
  { name: "cargo fmt", cmd: "cargo", args: ["fmt", "--check"], cwd: "src-tauri" },
];

if (!rapide) {
  steps.push(
    {
      name: "vitest",
      cmd: node,
      args: [
        "node_modules/vitest/vitest.mjs",
        "run",
        "--coverage",
        "--reporter=default",
        "--reporter=json",
        `--outputFile=${vitestJson}`,
      ],
      after: () =>
        combine(
          checkFloor(
            "vitest",
            readVitestReport(readFileSync(vitestJson, "utf-8")).passed,
            floors.vitest
          ),
          checkCoverage(
            "couverture ts",
            readCoverageSummary(readFileSync(coverageSummary, "utf-8")).lines,
            floors.couverture_ts
          )
        ),
    },
    { name: "build", cmd: node, args: ["node_modules/vite/bin/vite.js", "build"] },
    { name: "budget", cmd: node, args: ["scripts/bundle-budget.mjs"] },
    {
      name: "clippy",
      cmd: "cargo",
      args: ["clippy", "--all-targets", "-j", "2", "--", "-D", "warnings"],
      cwd: "src-tauri",
      env: rustEnv,
    },
    {
      name: "cargo test",
      cmd: "cargo",
      args: ["test", "-j", "2"],
      cwd: "src-tauri",
      env: rustEnv,
      capture: true,
      after: (stdout) => checkFloor("cargo", parseCargoTests(stdout).passed, floors.cargo),
    }
  );
}

const results = [];
let failed = null;
for (const step of steps) {
  const started = Date.now();
  console.log(`\n[verify] ${step.name}`);
  const r = spawnSync(step.cmd, step.args, {
    cwd: step.cwd,
    env: step.env ?? process.env,
    encoding: "utf-8",
    stdio: step.capture ? ["ignore", "pipe", "inherit"] : "inherit",
    maxBuffer: 256 * 1024 * 1024,
  });
  if (step.capture) process.stdout.write(r.stdout ?? "");
  const seconds = ((Date.now() - started) / 1000).toFixed(1);
  const { ok, detail } = judgeStep(r, step.after);
  results.push({ name: step.name, ok, seconds, detail });
  if (!ok) {
    failed = step.name;
    break;
  }
}

console.log("\n[verify] bilan");
for (const r of results) {
  console.log(
    `  ${r.ok ? "ok    " : "ECHEC "} ${r.name.padEnd(11)} ${r.seconds.padStart(7)} s  ${r.detail}`
  );
}
const after = gitState();
const tree = after.commit === before.commit && after.tree === before.tree ? before.tree : "modifie";
const line = [
  new Date().toISOString(),
  rapide ? "rapide" : "complet",
  `commit=${before.commit}`,
  `arbre=${tree}`,
  ...results.map((r) => `${r.name}=${r.seconds}s${r.ok ? "" : "(ECHEC)"}`),
  failed ? `ECHEC:${failed}` : "OK",
].join("\t");
appendFileSync("logs/verify.log", `${line}\n`);
if (failed) {
  console.error(`\n[verify] ECHEC à l'étape ${failed}`);
  process.exit(1);
}
console.log(`\n[verify] OK (${rapide ? "niveau rapide" : "niveau complet"})`);
