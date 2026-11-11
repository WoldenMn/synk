const CARGO_RESULT = /test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored/g;

export function parseCargoTests(output) {
  const total = { passed: 0, failed: 0, ignored: 0, suites: 0 };
  for (const m of output.matchAll(CARGO_RESULT)) {
    total.passed += Number(m[1]);
    total.failed += Number(m[2]);
    total.ignored += Number(m[3]);
    total.suites += 1;
  }
  return total;
}

export function readVitestReport(json) {
  const report = JSON.parse(json);
  const total = report.numTotalTests;
  const passed = report.numPassedTests;
  const failed = report.numFailedTests;
  if (![total, passed, failed].every(Number.isInteger)) {
    throw new Error("rapport vitest illisible : compteurs absents");
  }
  return { total, passed, failed };
}

export function checkFloor(name, value, floor) {
  if (!Number.isInteger(floor) || floor <= 0) {
    return {
      ok: false,
      message: `${name} : plancher absent ou illisible dans scripts/verify-floors.json`,
    };
  }
  if (!Number.isInteger(value)) {
    return { ok: false, message: `${name} : nombre de tests illisible` };
  }
  if (value < floor) {
    return {
      ok: false,
      message: `${name} : ${value} tests exécutés, sous le plancher de ${floor}`,
    };
  }
  if (value > floor) {
    return {
      ok: true,
      message: `${name} : ${value} tests, plancher de ${floor} à relever dans scripts/verify-floors.json`,
    };
  }
  return { ok: true, message: `${name} : ${value} tests, plancher atteint` };
}

export function judgeStep(result, after) {
  if (result.error) {
    return { ok: false, detail: result.error.message };
  }
  if (result.status !== 0) {
    return { ok: false, detail: `code ${result.status}` };
  }
  if (!after) {
    return { ok: true, detail: "code 0" };
  }
  try {
    const verdict = after(result.stdout ?? "");
    return { ok: verdict.ok, detail: verdict.message };
  } catch (e) {
    return { ok: false, detail: e.message };
  }
}

const TREE_SHA = /^[0-9a-f]{40}$/;

export function isTreeVerified(log, tree) {
  if (!TREE_SHA.test(tree)) return false;
  return log.split(/\r?\n/).some((raw) => {
    const fields = raw.split("\t");
    return fields[1] === "complet" && fields.at(-1) === "OK" && fields.includes(`arbre=${tree}`);
  });
}

const ZERO_SHA = /^0+$/;

export function pushedMainCommits(stdin) {
  return stdin
    .split(/\r?\n/)
    .map((raw) => raw.trim().split(/\s+/))
    .filter(
      ([, localSha, remoteRef]) =>
        remoteRef === "refs/heads/main" && localSha && !ZERO_SHA.test(localSha)
    )
    .map(([, localSha]) => localSha);
}

export function readCoverageSummary(json) {
  const pct = JSON.parse(json)?.total?.lines?.pct;
  if (typeof pct !== "number") {
    throw new Error("resume de couverture illisible : total.lines.pct absent");
  }
  return { lines: pct };
}

export function checkCoverage(name, pct, floor) {
  if (typeof floor !== "number" || !(floor > 0)) {
    return {
      ok: false,
      message: `${name} : plancher absent ou illisible dans scripts/verify-floors.json`,
    };
  }
  if (pct < floor) {
    return { ok: false, message: `${name} : ${pct} % des lignes, sous le plancher de ${floor} %` };
  }
  if (pct >= floor + 1) {
    return { ok: true, message: `${name} : ${pct} % des lignes, plancher de ${floor} % à relever` };
  }
  return { ok: true, message: `${name} : ${pct} % des lignes, plancher atteint` };
}

export function combine(...checks) {
  return { ok: checks.every((c) => c.ok), message: checks.map((c) => c.message).join(" ; ") };
}

export function summarizeLlvmCov(json, prefix) {
  const data = JSON.parse(json)?.data?.[0];
  const total = data?.totals?.lines?.percent;
  if (typeof total !== "number") {
    throw new Error("export llvm-cov illisible : totals.lines.percent absent");
  }
  const files = (data.files ?? []).filter((f) => f.filename.replaceAll("\\", "/").includes(prefix));
  if (files.length === 0) throw new Error(`aucun fichier sous ${prefix} dans l'export llvm-cov`);
  const count = files.reduce((a, f) => a + f.summary.lines.count, 0);
  const covered = files.reduce((a, f) => a + f.summary.lines.covered, 0);
  return {
    total: Math.round(total * 10) / 10,
    prefix: Math.round((covered / count) * 1000) / 10,
  };
}
