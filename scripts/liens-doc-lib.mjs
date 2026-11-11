import { dirname, posix } from "node:path";

export function stripCode(markdown) {
  return markdown.replace(/^(```|~~~)[^\n]*\n[\s\S]*?^\1[^\n]*$/gm, (m) =>
    m.replace(/[^\n]/g, " ")
  );
}

export function extractLinks(markdown) {
  const text = stripCode(markdown);
  const links = [];
  const lines = text.split("\n");
  lines.forEach((line, i) => {
    const bare = line.replace(/`[^`]*`/g, (m) => " ".repeat(m.length));
    for (const m of bare.matchAll(/\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g)) {
      links.push({ target: m[1], line: i + 1 });
    }
    for (const m of bare.matchAll(/<a\s+[^>]*href="([^"]+)"/g)) {
      links.push({ target: m[1], line: i + 1 });
    }
  });
  return links;
}

export function slugOf(heading) {
  return heading
    .trim()
    .toLowerCase()
    .replace(/`/g, "")
    .replace(/[^\p{L}\p{N}\s_-]/gu, "")
    .replace(/\s/g, "-");
}

export function anchorsOf(markdown) {
  const text = stripCode(markdown);
  const anchors = new Set();
  for (const m of text.matchAll(/^#{1,6}\s+(.+?)\s*#*\s*$/gm)) anchors.add(slugOf(m[1]));
  for (const m of text.matchAll(/\bid="([^"]+)"/g)) anchors.add(m[1]);
  return anchors;
}

const EXTERNAL = /^([a-z][a-z0-9+.-]*:|\/\/)/i;

export function resolveTarget(fromFile, target) {
  const [rawPath, anchor] = target.split("#");
  if (rawPath === "") return { path: fromFile, anchor: anchor ?? null };
  const path = decodeURI(rawPath);
  return { path: posix.normalize(posix.join(dirname(fromFile), path)), anchor: anchor ?? null };
}

export function brokenLinks(files, exists, readText, skipPrefixes = []) {
  const problems = [];
  for (const file of files) {
    for (const { target, line } of extractLinks(readText(file))) {
      if (EXTERNAL.test(target)) continue;
      const { path, anchor } = resolveTarget(file, target);
      if (skipPrefixes.some((p) => path === p || path.startsWith(`${p}/`))) continue;
      if (!exists(path)) {
        problems.push(`${file}:${line} : ${target} (fichier absent : ${path})`);
        continue;
      }
      if (anchor && path.endsWith(".md") && !anchorsOf(readText(path)).has(anchor.toLowerCase())) {
        problems.push(`${file}:${line} : ${target} (ancre absente dans ${path})`);
      }
    }
  }
  return problems;
}
