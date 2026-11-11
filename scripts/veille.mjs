import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import {
  applyExceptions,
  breakingBehind,
  cargoAuditFindings,
  intendanceFindings,
  msrvProblem,
  npmAuditFindings,
  npmMajorFindings,
  snapshotCovers,
} from "./veille-lib.mjs";

const args = process.argv.slice(2);
const option = (name) => args.find((a) => a.startsWith(`--${name}=`))?.slice(name.length + 3);
const today = new Date().toLocaleDateString("sv-SE");
const problems = [];
const notes = [];
const LOCKFILES = ["package-lock.json", "src-tauri/Cargo.lock"];

const npmCli =
  process.env.npm_execpath ??
  join(dirname(process.execPath), "node_modules", "npm", "bin", "npm-cli.js");

function run(cmd, cmdArgs, cwd = ".") {
  const r = spawnSync(cmd, cmdArgs, { cwd, encoding: "utf8", maxBuffer: 256 * 1024 * 1024 });
  if (r.error) throw new Error(`${cmd} ${cmdArgs.join(" ")} : ${r.error.message}`);
  return r;
}

function json(cmd, cmdArgs, cwd) {
  const r = run(cmd, cmdArgs, cwd);
  try {
    return JSON.parse(r.stdout);
  } catch {
    const out = (r.stderr || r.stdout).slice(0, 400);
    throw new Error(
      `${cmd} ${cmdArgs.join(" ")} n'a pas rendu de JSON (code ${r.status}) : ${out}`
    );
  }
}

const npmJson = (npmArgs) => json(process.execPath, [npmCli, ...npmArgs]);

function lockfilesChangedAt() {
  const dirty = run("git", ["status", "--porcelain", "--", ...LOCKFILES]).stdout.trim();
  if (dirty) return new Date().toISOString();
  return run("git", ["log", "-1", "--format=%cI", "--", ...LOCKFILES]).stdout.trim();
}

function latestIntendance() {
  const given = option("intendance");
  if (given) return given;
  const dir =
    process.env.VEILLE_INTENDANCE_DIR ??
    join(homedir(), "Desktop", "Routines IA", "intendance", "donnees");
  if (!existsSync(dir)) return null;
  const files = readdirSync(dir).filter((f) => /^veille-\d{4}-\d{2}-\d{2}\.json$/.test(f));
  return files.length ? join(dir, files.sort().at(-1)) : null;
}

function intendanceSection() {
  const path = latestIntendance();
  if (!path) {
    notes.push("Intendance : aucun rapport de veille trouve, lecture sautee");
    return [];
  }
  const report = JSON.parse(readFileSync(path, "utf8"));
  const age = Math.round((Date.now() - Date.parse(report.genere_le)) / 3600000);
  const all = intendanceFindings(report, "Synk");
  if (!snapshotCovers(report.genere_le, lockfilesChangedAt())) {
    notes.push(
      `Intendance : rapport du ${report.genere_le} (${age} h) anterieur aux verrous, ${all.length} faille(s) haute(s) ignoree(s) ; la veille de la nuit prochaine les relira`
    );
    return [];
  }
  notes.push(
    `Intendance : rapport du ${report.genere_le} (${age} h), ${all.length} faille(s) haute(s) ou critique(s)`
  );
  return all.map((f) => ({ ...f, source: "intendance" }));
}

async function crateFindings(synk) {
  const out = [];
  const direct = [
    ...new Map(synk.dependencies.filter((d) => d.source).map((d) => [d.name, d.req])),
  ];
  for (const [name, req] of direct) {
    const res = await fetch(`https://crates.io/api/v1/crates/${name}`, {
      headers: { "User-Agent": "synk-veille (usage local)" },
    });
    if (!res.ok) throw new Error(`crates.io : ${name} -> HTTP ${res.status}`);
    const latest = (await res.json()).crate.max_stable_version;
    if (breakingBehind(req, latest)) {
      out.push({ id: `majeure:${name}`, paquet: name, actuelle: req, derniere: latest });
    }
  }
  return out;
}

const exceptions = JSON.parse(readFileSync("scripts/veille-exceptions.json", "utf8"));
const auditFile = option("audit-npm");
const npmAudit = auditFile
  ? JSON.parse(readFileSync(auditFile, "utf8"))
  : npmJson(["audit", "--json"]);
const findings = [
  ...npmAuditFindings(npmAudit),
  ...npmAuditFindings(npmJson(["audit", "--omit=dev", "--json"])),
  ...npmMajorFindings(npmJson(["outdated", "--json"])),
  ...cargoAuditFindings(json("cargo", ["audit", "--json"], "src-tauri")),
  ...intendanceSection(),
];

const metadata = json(
  "cargo",
  ["metadata", "--format-version", "1", "--filter-platform", "x86_64-pc-windows-msvc"],
  "src-tauri"
);
const synk = metadata.packages.find((p) => p.name === "synk");
if (!synk) throw new Error("cargo metadata : paquet synk introuvable");
findings.push(...(await crateFindings(synk)));

const msrv = msrvProblem(
  synk.rust_version ?? "0",
  metadata.packages.map((p) => p.rust_version).filter(Boolean)
);
if (msrv) problems.push(msrv);

const deny = run("cargo", ["deny", "check", "licenses", "bans"], "src-tauri");
if (deny.status !== 0) {
  problems.push(
    `cargo deny : ${(deny.stderr || deny.stdout).trim().split("\n").slice(-15).join("\n")}`
  );
}
const machete = run("cargo", ["machete"], "src-tauri");
if (machete.status !== 0) problems.push(`cargo machete : ${machete.stdout.trim()}`);

const unique = [...new Map(findings.map((f) => [f.id, f])).values()];
const verdict = applyExceptions(unique, exceptions, today);
for (const f of verdict.failing) {
  problems.push(
    f.exception
      ? `exception echue le ${f.exception.echeance} : ${f.id} (${f.paquet})`
      : `sans exception : ${f.id} (${f.paquet}) ${f.titre ?? `${f.actuelle} -> ${f.derniere}`}`
  );
}

mkdirSync("logs", { recursive: true });
const report = [
  `# Veille du ${today}`,
  "",
  `## Problemes (${problems.length})`,
  ...problems.map((p) => `- ${p}`),
  "",
  `## Excuses (${verdict.excused.length})`,
  ...verdict.excused.map(
    (f) => `- ${f.id} (${f.paquet}) : ${f.exception.motif}, jusqu'au ${f.exception.echeance}`
  ),
  "",
  `## Notes (${notes.length})`,
  ...notes.map((n) => `- ${n}`),
].join("\n");
writeFileSync(`logs/veille-${today}.md`, report);
console.log(report);
process.exit(problems.length === 0 ? 0 : 1);
