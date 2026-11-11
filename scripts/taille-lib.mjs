export function productionLines(path, text) {
  const lines = text.split("\n");
  if (lines.at(-1) === "") lines.pop();
  if (!path.endsWith(".rs")) return lines.length;
  const cut = lines.findIndex((l) => l.trim() === "#[cfg(test)]");
  return cut === -1 ? lines.length : cut;
}

export function sizeProblems(sizes, max, exceptions) {
  const problems = [];
  const byPath = new Map(sizes.map((s) => [s.path, s.lines]));
  for (const { path, lines } of sizes) {
    if (lines > max && !exceptions[path]) {
      problems.push(`${path} : ${lines} lignes, au-dela de ${max}`);
    }
  }
  for (const path of Object.keys(exceptions)) {
    if (!byPath.has(path)) problems.push(`${path} : exception perimee (fichier absent)`);
    else if (byPath.get(path) <= max) {
      problems.push(`${path} : exception perimee (${byPath.get(path)} lignes)`);
    }
  }
  return problems;
}
