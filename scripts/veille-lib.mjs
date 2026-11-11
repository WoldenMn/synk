const SERIOUS = new Set(["high", "critical"]);
const SERIOUS_FR = new Set(["haute", "critique"]);

export function npmAuditFindings(report) {
  const found = new Map();
  for (const vuln of Object.values(report.vulnerabilities ?? {})) {
    for (const via of vuln.via ?? []) {
      if (typeof via !== "object" || !SERIOUS.has(via.severity)) continue;
      const id = (via.url ?? "").split("/").pop() || `npm-${via.source}`;
      found.set(id, { id, paquet: via.name, severite: via.severity, titre: via.title });
    }
  }
  return [...found.values()];
}

export function cargoAuditFindings(report) {
  const out = (report.vulnerabilities?.list ?? []).map((v) => ({
    id: v.advisory.id,
    paquet: v.package.name,
    version: v.package.version,
    genre: "vulnerabilite",
    titre: v.advisory.title,
  }));
  for (const [genre, list] of Object.entries(report.warnings ?? {})) {
    for (const w of list ?? []) {
      out.push({
        id: w.advisory?.id ?? `${genre}:${w.package.name}`,
        paquet: w.package.name,
        version: w.package.version,
        genre,
        titre: w.advisory?.title ?? genre,
      });
    }
  }
  return out;
}

function parts(version) {
  return version
    .replace(/^[^0-9]*/, "")
    .split(".")
    .map((n) => Number.parseInt(n, 10) || 0);
}

export function breakingBehind(current, latest) {
  const [ca = 0, cb = 0] = parts(current);
  const [la = 0, lb = 0] = parts(latest);
  if (ca === 0 && la === 0) return lb > cb;
  return la > ca;
}

export function npmMajorFindings(outdated) {
  return Object.entries(outdated)
    .filter(([, info]) => info.current && info.latest && breakingBehind(info.current, info.latest))
    .map(([paquet, info]) => ({
      id: `majeure:${paquet}`,
      paquet,
      actuelle: info.current,
      derniere: info.latest,
    }));
}

function compare(a, b) {
  const pa = parts(a);
  const pb = parts(b);
  for (let i = 0; i < 3; i++) {
    const d = (pa[i] ?? 0) - (pb[i] ?? 0);
    if (d !== 0) return d;
  }
  return 0;
}

export function msrvProblem(declared, versions) {
  const highest = versions.reduce((max, v) => (compare(v, max) > 0 ? v : max), "0");
  return compare(declared, highest) < 0
    ? `rust-version ${declared} sous le plancher reel de l'arbre (${highest})`
    : null;
}

export function applyExceptions(findings, exceptions, today) {
  const byId = new Map(exceptions.filter((e) => e.motif && e.motif.trim()).map((e) => [e.id, e]));
  const failing = [];
  const excused = [];
  const expired = [];
  for (const f of findings) {
    const exception = byId.get(f.id);
    if (!exception) failing.push(f);
    else if (exception.echeance < today) {
      expired.push({ ...f, exception });
      failing.push({ ...f, exception });
    } else excused.push({ ...f, exception });
  }
  return { failing, excused, expired };
}

export function intendanceFindings(report, projet) {
  const found = new Map();
  for (const c of report.constats ?? []) {
    if (c.genre !== "faille" || c.composant !== projet || !SERIOUS_FR.has(c.severite)) continue;
    const alias = c.identifiants ?? [];
    const id =
      alias.find((a) => /^(GHSA|RUSTSEC)-/.test(a)) ?? alias[0] ?? `${c.ecosysteme}:${c.paquet}`;
    found.set(id, { id, paquet: c.paquet, severite: c.severite, titre: c.titre, alias });
  }
  return [...found.values()];
}

export function snapshotCovers(generatedAt, lockfilesChangedAt) {
  const generated = Date.parse(generatedAt);
  const changed = Date.parse(lockfilesChangedAt);
  if (Number.isNaN(generated) || Number.isNaN(changed)) return false;
  return generated >= changed;
}
