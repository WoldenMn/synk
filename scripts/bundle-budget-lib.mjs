import { gzipSync } from "node:zlib";

function attribute(attrs, name) {
  const match = attrs.match(new RegExp(`\\b${name}="([^"]+)"`));
  return match ? match[1] : null;
}

export function criticalAssets(html) {
  const entry = [];
  for (const [, attrs] of html.matchAll(/<script\b([^>]*)>/g)) {
    const src = attribute(attrs, "src");
    if (attribute(attrs, "type") === "module" && src) entry.push(src);
  }
  const preloads = [];
  for (const [, attrs] of html.matchAll(/<link\b([^>]*)>/g)) {
    const href = attribute(attrs, "href");
    if (attribute(attrs, "rel") === "modulepreload" && href) preloads.push(href);
  }
  return { entry, preloads };
}

export function staticImports(js) {
  const found = [];
  for (const [, file] of js.matchAll(/\b(?:from|import)\s*["']\.\/([^"']+\.js)["']/g)) {
    found.push(file);
  }
  return found;
}

function dynamicChunk(js, page) {
  const quote = "[\"'" + String.fromCharCode(96) + "]";
  const pattern = new RegExp(`import\\(\\s*${quote}\\./(${page}-[A-Za-z0-9_-]+\\.js)${quote}`);
  return js.match(pattern)?.[1] ?? null;
}

export function firstScreenAssets({ entry, page, read }) {
  const entryJs = read(entry);
  if (entryJs === undefined) throw new Error(`point d'entree illisible : ${entry}`);
  const pageChunk = dynamicChunk(entryJs, page);
  if (!pageChunk) throw new Error(`chunk de la page ${page} introuvable dans ${entry}`);
  const seen = new Set();
  const queue = [entry, pageChunk];
  while (queue.length > 0) {
    const file = queue.shift();
    if (seen.has(file)) continue;
    const js = read(file);
    if (js === undefined) throw new Error(`chunk du premier ecran illisible : ${file}`);
    seen.add(file);
    queue.push(...staticImports(js));
  }
  return [...seen];
}

export function gzipKb(buffer) {
  return Math.round((gzipSync(buffer).length / 1024) * 10) / 10;
}

export function checkBudget(measure, budget) {
  const problems = [];
  if (measure.entryKb > budget.entryGzipKb) {
    problems.push(`chunk d'entree : ${measure.entryKb} ko gzip, plafond ${budget.entryGzipKb} ko`);
  }
  if (measure.criticalKb > budget.criticalGzipKb) {
    problems.push(
      `chemin critique : ${measure.criticalKb} ko gzip, plafond ${budget.criticalGzipKb} ko`
    );
  }
  if (budget.firstScreenGzipKb === undefined) {
    problems.push("premier ecran : plafond absent de bundle-budget.json");
  } else if (measure.firstScreenKb > budget.firstScreenGzipKb) {
    problems.push(
      `premier ecran : ${measure.firstScreenKb} ko gzip, plafond ${budget.firstScreenGzipKb} ko`
    );
  }
  return { ok: problems.length === 0, problems };
}
