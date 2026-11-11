import { createHash } from "node:crypto";
import { spawn, spawnSync } from "node:child_process";
import {
  appendFileSync,
  copyFileSync,
  createWriteStream,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { freemem } from "node:os";
import { join } from "node:path";
import {
  bumpAll,
  changelogProblem,
  compareSemver,
  installerProblems,
  isTransientParcours,
  journalLine,
  parseArgs,
  parseShaLine,
  previousInstaller,
  shaLine,
  stepPlan,
} from "./release-lib.mjs";
import { VERSION_FILES } from "./version-lib.mjs";

const DEST = "D:\\Sauvegardes\\Synk\\installeurs";
const BUNDLE = "src-tauri/target/release/bundle/nsis";
const NODE = process.execPath;
const NPM = process.env.npm_execpath;

const args = parseArgs(process.argv.slice(2));
if (args.error) {
  console.error(`[release] ${args.error}`);
  process.exit(1);
}
const { version, simulation } = args;
const tag = simulation ? `${version}-simulation` : version;
const logDir = join("logs", `release-${tag}`);
mkdirSync(logDir, { recursive: true });

const steps = [];
let minFree = freemem();
const sampler = setInterval(() => (minFree = Math.min(minFree, freemem())), 5000);

function run(id, cmd, cmdArgs, env = {}) {
  console.log(`\n[release] ${id}`);
  return new Promise((resolve, reject) => {
    const sink = createWriteStream(join(logDir, `${id}.out`), { flags: "a" });
    const started = Date.now();
    const useShell = !NPM && cmd === "npm";
    const child = spawn(cmd, cmdArgs, {
      env: { ...process.env, ...env },
      shell: useShell,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let text = "";
    const take = (chunk) => {
      text += chunk;
      sink.write(chunk);
      process.stdout.write(chunk);
    };
    child.stdout.on("data", take);
    child.stderr.on("data", take);
    child.on("error", reject);
    child.on("close", (code) => {
      sink.end();
      resolve({ code, text, seconds: Math.round((Date.now() - started) / 1000) });
    });
  });
}

function npm(id, script, extra = [], env = {}) {
  const base = NPM ? [NODE, [NPM]] : ["npm", []];
  return run(id, base[0], [...base[1], "run", script, ...extra], env);
}

async function stage(id, action) {
  const started = Date.now();
  await action();
  steps.push({ id, seconds: Math.round((Date.now() - started) / 1000) });
}

function ok(result, label) {
  if (result.code !== 0)
    throw new Error(`${label} en echec (code ${result.code}), trace dans ${logDir}`);
}

function git(...gitArgs) {
  const r = spawnSync("git", gitArgs, { encoding: "utf8" });
  if (r.status !== 0) throw new Error(`git ${gitArgs.join(" ")} : ${r.stderr}`);
  return r.stdout.trim();
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex").toUpperCase();
}

function fileVersion(path) {
  const r = spawnSync(
    "powershell",
    ["-NoProfile", "-Command", `(Get-Item -LiteralPath '${path}').VersionInfo.FileVersion`],
    { encoding: "utf8" }
  );
  return r.status === 0 ? r.stdout.trim() : null;
}

const handlers = {
  async controles() {
    const branch = git("rev-parse", "--abbrev-ref", "HEAD");
    if (branch === "main" || branch === "HEAD") {
      throw new Error(`la release se prepare sur une branche, pas sur ${branch}`);
    }
    if (git("status", "--porcelain", "--untracked-files=all") !== "") {
      throw new Error("arbre de travail non propre : commiter ou ecarter avant");
    }
    const current = JSON.parse(readFileSync("package.json", "utf8")).version;
    const order = compareSemver(version, current);
    if (simulation ? order < 0 : order <= 0) {
      throw new Error(`la version ${version} ne depasse pas la version courante ${current}`);
    }
    const problem = changelogProblem(readFileSync("CHANGELOG.md", "utf8"), version);
    if (problem) throw new Error(`CHANGELOG : ${problem}`);
    if (!simulation) {
      if (!existsSync(DEST)) throw new Error(`${DEST} absent : disque D: non monte ?`);
      if (existsSync(join(DEST, `Synk_${version}_x64-setup.exe`))) {
        throw new Error(`Synk_${version}_x64-setup.exe existe deja dans ${DEST} : jamais ecrase`);
      }
    }
  },

  async veille() {
    ok(await npm("veille", "veille"), "veille des dependances");
  },

  async version() {
    const read = (p) => readFileSync(p, "utf8");
    const { texts, wrong } = bumpAll(read, version);
    if (Object.keys(wrong).length > 0) {
      throw new Error(`emplacements non montes : ${JSON.stringify(wrong)}`);
    }
    if (simulation) {
      console.log(`[release] simulation : montee a ${version} verifiee en memoire, rien d'ecrit`);
      return;
    }
    for (const path of VERSION_FILES) {
      const crlf = read(path).includes("\r\n");
      writeFileSync(path, crlf ? texts[path].replace(/\n/g, "\r\n") : texts[path]);
    }
    git("add", ...VERSION_FILES);
    git("commit", "-m", `chore(release): ${version}`);
  },

  async verify() {
    ok(await npm("verify", "verify"), "verification complete");
    const tree = git("rev-parse", "HEAD^{tree}");
    const last = readFileSync("logs/verify.log", "utf8").trim().split("\n").at(-1) ?? "";
    if (!last.includes(`arbre=${tree}`) || !last.endsWith("OK") || !last.includes("complet")) {
      throw new Error(`derniere ligne de logs/verify.log sans l'arbre ${tree} : ${last}`);
    }
  },

  async couverture() {
    ok(await npm("couverture", "couverture:rust"), "couverture Rust");
  },

  async parcours() {
    let result = await npm("parcours", "parcours");
    if (result.code !== 0 && isTransientParcours(result.text)) {
      console.log(
        "[release] parcours : silence CDP ou repli de l'IPC au demarrage, un second essai"
      );
      result = await npm("parcours-essai2", "parcours");
    }
    ok(result, "parcours des 19 pages");
  },

  async budget() {
    ok(await npm("budget", "budget"), "budget de taille");
  },

  async build() {
    state.buildStart = Date.now();
    ok(await npm("build", "tauri", ["--", "build"], { CARGO_BUILD_JOBS: "1" }), "build NSIS");
  },

  async installeur() {
    const name = `Synk_${version}_x64-setup.exe`;
    const path = join(BUNDLE, name);
    if (!existsSync(path)) throw new Error(`${name} introuvable dans ${BUNDLE}`);
    const stat = statSync(path);
    const before = existsSync(DEST) ? previousInstaller(readdirSync(DEST), version) : null;
    const problems = installerProblems({
      name,
      size: stat.size,
      mtimeMs: stat.mtimeMs,
      buildStartMs: state.buildStart,
      fileVersion: fileVersion(path),
      version,
      previousSize: before ? statSync(join(DEST, before)).size : null,
    });
    if (problems.length > 0) throw new Error(problems.join(" ; "));
    state.installer = { name, path, bytes: stat.size };
  },

  async copie() {
    const { name, path, bytes } = state.installer;
    const target = join(DEST, name);
    copyFileSync(path, target);
    const hash = sha256(path);
    writeFileSync(`${target}.sha256`, shaLine(hash, name));
    const written = parseShaLine(readFileSync(`${target}.sha256`, "utf8"));
    if (!written || written.hash !== hash || sha256(target) !== hash) {
      throw new Error("copie de l'installeur ou somme illisible : empreintes differentes");
    }
    state.sha = hash;
    state.bytes = bytes;
  },
};

const state = { buildStart: 0, installer: null, sha: null, bytes: null };

try {
  for (const { id } of stepPlan({ simulation })) {
    await stage(id, handlers[id]);
  }
} catch (e) {
  clearInterval(sampler);
  console.error(`\n[release] ARRET : ${e.message}`);
  process.exit(1);
}
clearInterval(sampler);

const line = journalLine({
  date: new Date().toISOString(),
  version,
  simulation,
  steps,
  minFreeMb: Math.round(minFree / 1048576),
  sha: state.sha,
  bytes: state.bytes,
  tree: git("rev-parse", "HEAD^{tree}"),
});
const journal = simulation
  ? join("logs", "journal-des-releases.txt")
  : join(DEST, "journal-des-releases.txt");
appendFileSync(journal, line);
console.log(
  `\n[release] ${simulation ? "simulation" : "release"} ${version} finie, journal : ${journal}`
);
for (const s of steps) console.log(`  ${s.id.padEnd(11)} ${String(s.seconds).padStart(6)} s`);
if (!simulation) {
  console.log(`[release] ${join(DEST, state.installer.name)}`);
  console.log(`[release] sha256 ${state.sha}`);
  console.log(
    "[release] aucune etiquette posee, rien de pousse : a lancer par toi apres la fusion"
  );
  console.log(`[release]   git tag v${version}`);
  console.log("[release] installation : au user");
}
