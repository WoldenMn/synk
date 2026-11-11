import { bumpText, disagreements, isSemver, readVersions, VERSION_FILES } from "./version-lib.mjs";

export const MAX_GROWTH = 0.1;
export const MIN_INSTALLER_BYTES = 1_000_000;

export function parseArgs(argv) {
  const options = argv.filter((a) => a.startsWith("--"));
  const unknown = options.find((a) => a !== "--simulation");
  if (unknown) return { error: `option inconnue : ${unknown}` };
  const version = argv.find((a) => !a.startsWith("--"));
  if (!version) return { error: "usage : npm run release -- X.Y.Z [--simulation]" };
  if (!isSemver(version)) return { error: `version mal formee : ${version}` };
  return { version, simulation: options.includes("--simulation") };
}

export function compareSemver(a, b) {
  const left = a.split(".").map(Number);
  const right = b.split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    if (left[i] !== right[i]) return left[i] > right[i] ? 1 : -1;
  }
  return 0;
}

export function changelogProblem(text, version) {
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  const pending = lines.findIndex((l) => l.startsWith("## [Non publi"));
  if (pending !== -1) {
    const next = lines.findIndex((l, i) => i > pending && l.startsWith("## ["));
    const body = lines.slice(pending + 1, next === -1 ? undefined : next);
    if (body.some((l) => l.trim() !== "")) {
      return "la section Non publié contient encore des notes : les dater avant la release";
    }
  }
  const heading = lines.find((l) => l.startsWith(`## [${version}]`));
  if (!heading) return `CHANGELOG sans section pour ${version}`;
  if (!/^## \[[^\]]+\] - \d{4}-\d{2}-\d{2}$/.test(heading)) {
    return `la section ${version} du CHANGELOG n'a pas de date AAAA-MM-JJ`;
  }
  return null;
}

export function shaLine(hash, name) {
  return `${hash.toUpperCase()}  ${name}\n`;
}

export function parseShaLine(text) {
  const match = text.trim().match(/^([0-9A-Fa-f]{64}) {2}(.+)$/);
  return match ? { hash: match[1].toUpperCase(), name: match[2] } : null;
}

const INSTALLER = /^Synk_(\d+\.\d+\.\d+)_x64-setup\.exe$/;

export function previousInstaller(names, version) {
  const found = names
    .map((name) => ({ name, v: name.match(INSTALLER)?.[1] }))
    .filter((e) => e.v && compareSemver(e.v, version) < 0)
    .sort((a, b) => compareSemver(b.v, a.v));
  return found[0]?.name ?? null;
}

export function installerProblems(i) {
  const problems = [];
  if (!(i.size >= MIN_INSTALLER_BYTES)) {
    problems.push(`taille de ${i.name} anormale : ${i.size} octets`);
  }
  if (!(i.mtimeMs >= i.buildStartMs)) {
    problems.push(
      `${i.name} est anterieur au build : ce n'est pas le fichier qui vient d'etre construit`
    );
  }
  if (i.fileVersion !== i.version) {
    problems.push(`version de fichier ${i.fileVersion ?? "illisible"}, attendue ${i.version}`);
  }
  if (i.previousSize && i.size > i.previousSize * (1 + MAX_GROWTH)) {
    const pct = (((i.size - i.previousSize) / i.previousSize) * 100).toFixed(1);
    problems.push(
      `hausse de poids de ${pct} % sur la version precedente, au-dela de ${MAX_GROWTH * 100} %`
    );
  }
  return problems;
}

export function isCdpSilence(output) {
  return /aucune reponse CDP/.test(output);
}

const IPC_FALLBACK_LINE = /^\[parcours\] demarrage ou navigation : IPC replie sur postMessage/;
const PARCOURS_SUMMARY_LINE = /^\[parcours\] \d+ pages, \d+ probleme/;

export function isTransientParcours(output) {
  if (isCdpSilence(output)) return true;
  const lines = output.split(/\r?\n/);
  if (!lines.some((l) => IPC_FALLBACK_LINE.test(l))) return false;
  return !lines.some(
    (l) =>
      /^ECHEC\b/.test(l) ||
      (/^\[parcours\] /.test(l) && !IPC_FALLBACK_LINE.test(l) && !PARCOURS_SUMMARY_LINE.test(l))
  );
}

const STEPS = [
  { id: "controles", real: false },
  { id: "veille", real: false },
  { id: "version", real: false },
  { id: "verify", real: false },
  { id: "couverture", real: false },
  { id: "parcours", real: false },
  { id: "budget", real: false },
  { id: "build", real: true },
  { id: "installeur", real: true },
  { id: "copie", real: true },
];

export function stepPlan({ simulation }) {
  return STEPS.filter((s) => !simulation || !s.real);
}

export function bumpAll(read, version) {
  const before = readVersions(read);
  const apart = disagreements(before);
  if (Object.keys(apart).length > 0) {
    throw new Error(`versions en desaccord avant la montee : ${JSON.stringify(before)}`);
  }
  const texts = Object.fromEntries(
    VERSION_FILES.map((p) => [p, bumpText(p, read(p).replace(/\r\n/g, "\n"), version)])
  );
  const after = readVersions((p) => texts[p]);
  const wrong = Object.fromEntries(Object.entries(after).filter(([, v]) => v !== version));
  return { texts, wrong };
}

export function journalLine(r) {
  return (
    [
      r.date,
      r.version,
      r.simulation ? "simulation" : "release",
      ...r.steps.map((s) => `${s.id}=${s.seconds}s`),
      `memoire_libre_min=${r.minFreeMb}Mo`,
      `arbre=${r.tree}`,
      ...(r.sha ? [`octets=${r.bytes}`, `sha256=${r.sha}`] : []),
    ].join("\t") + "\n"
  );
}
