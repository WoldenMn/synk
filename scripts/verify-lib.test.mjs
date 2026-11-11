import { describe, expect, it } from "vitest";
import {
  checkCoverage,
  checkFloor,
  combine,
  isTreeVerified,
  judgeStep,
  parseCargoTests,
  pushedMainCommits,
  readCoverageSummary,
  readVitestReport,
  summarizeLlvmCov,
} from "./verify-lib.mjs";

const CARGO_OK = [
  "running 137 tests",
  "test result: ok. 137 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.10s",
  "running 3 tests",
  "test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s",
].join("\n");

const CARGO_KO = [
  "test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s",
  "test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s",
].join("\n");

describe("parseCargoTests", () => {
  it("additionne les tests de tous les binaires", () => {
    expect(parseCargoTests(CARGO_OK)).toEqual({ passed: 139, failed: 0, ignored: 1, suites: 2 });
  });

  it("compte les échecs d'un binaire en FAILED", () => {
    expect(parseCargoTests(CARGO_KO)).toEqual({ passed: 14, failed: 1, ignored: 0, suites: 2 });
  });

  it("rend zéro suite quand la sortie ne contient aucun résultat", () => {
    expect(parseCargoTests("error: could not compile `synk`")).toEqual({
      passed: 0,
      failed: 0,
      ignored: 0,
      suites: 0,
    });
  });
});

describe("readVitestReport", () => {
  it("lit les compteurs du rapport JSON", () => {
    const json = JSON.stringify({ numTotalTests: 419, numPassedTests: 418, numFailedTests: 1 });
    expect(readVitestReport(json)).toEqual({ total: 419, passed: 418, failed: 1 });
  });

  it("refuse un rapport sans compteurs", () => {
    expect(() => readVitestReport("{}")).toThrow(/rapport vitest illisible/);
  });
});

describe("checkFloor", () => {
  it("passe au plancher exact", () => {
    expect(checkFloor("vitest", 419, 419).ok).toBe(true);
  });

  it("échoue sous le plancher et donne les deux nombres", () => {
    const r = checkFloor("vitest", 12, 419);
    expect(r.ok).toBe(false);
    expect(r.message).toMatch(/12 .*419/);
  });

  it("signale un plancher à relever sans échouer", () => {
    const r = checkFloor("cargo", 390, 378);
    expect(r.ok).toBe(true);
    expect(r.message).toMatch(/relever/);
  });

  it("refuse un plancher absent", () => {
    const r = checkFloor("vitest", 427, undefined);
    expect(r.ok).toBe(false);
    expect(r.message).toMatch(/plancher absent ou illisible/);
  });

  it("refuse un plancher qui n'est pas un entier positif", () => {
    expect(checkFloor("cargo", 378, "378").ok).toBe(false);
    expect(checkFloor("cargo", 378, 0).ok).toBe(false);
  });

  it("refuse un nombre de tests illisible", () => {
    const r = checkFloor("cargo", Number.NaN, 378);
    expect(r.ok).toBe(false);
    expect(r.message).toMatch(/illisible/);
  });
});

describe("judgeStep", () => {
  it("rend un échec sur un exécutable introuvable", () => {
    const r = judgeStep({ status: null, error: new Error("spawn cargo ENOENT") });
    expect(r).toEqual({ ok: false, detail: "spawn cargo ENOENT" });
  });

  it("rend un échec sur un code non nul sans lancer l'analyse", () => {
    let analysee = false;
    const r = judgeStep({ status: 2, stdout: "" }, () => {
      analysee = true;
      return { ok: true, message: "" };
    });
    expect(r).toEqual({ ok: false, detail: "code 2" });
    expect(analysee).toBe(false);
  });

  it("rend un succès sur un code nul sans analyse", () => {
    expect(judgeStep({ status: 0, stdout: "" })).toEqual({ ok: true, detail: "code 0" });
  });

  it("porte le verdict de l'analyse", () => {
    const r = judgeStep({ status: 0, stdout: "x" }, (stdout) => ({
      ok: false,
      message: `vu ${stdout}`,
    }));
    expect(r).toEqual({ ok: false, detail: "vu x" });
  });

  it("rend un échec quand l'analyse lève une erreur", () => {
    const r = judgeStep({ status: 0, stdout: "" }, () => {
      throw new Error("ENOENT: logs/vitest-report.json");
    });
    expect(r.ok).toBe(false);
    expect(r.detail).toMatch(/ENOENT/);
  });
});

const TREE = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const OTHER = "1111111111111111111111111111111111111111";
const line = (niveau, arbre, fin = "OK") =>
  ["2026-10-22T10:00:00.000Z", niveau, "commit=abc", `arbre=${arbre}`, "vitest=19.7s", fin].join(
    "\t"
  );

describe("isTreeVerified", () => {
  it("accepte un arbre passé au niveau complet sans échec", () => {
    const log = `${line("complet", OTHER)}\n${line("complet", TREE)}\n`;
    expect(isTreeVerified(log, TREE)).toBe(true);
  });

  it("refuse un arbre seulement passé au niveau rapide", () => {
    expect(isTreeVerified(line("rapide", TREE), TREE)).toBe(false);
  });

  it("refuse un arbre dont le passage complet a échoué", () => {
    expect(isTreeVerified(line("complet", TREE, "ECHEC:cargo test"), TREE)).toBe(false);
  });

  it("refuse un passage fait sur un arbre modifié", () => {
    expect(isTreeVerified(line("complet", "modifie"), "modifie")).toBe(false);
  });

  it("refuse les lignes d'avant D12, qui ne portent pas l'arbre", () => {
    const ancien = ["2026-10-22T09:50:23.372Z", "complet", "prettier=4.0s", "OK"].join("\t");
    expect(isTreeVerified(ancien, TREE)).toBe(false);
  });

  it("refuse un autre arbre", () => {
    expect(isTreeVerified(line("complet", OTHER), TREE)).toBe(false);
  });
});

describe("pushedMainCommits", () => {
  const ZERO = "0000000000000000000000000000000000000000";

  it("garde les commits poussés vers main", () => {
    const stdin = `refs/heads/main aaa refs/heads/main ${ZERO}\nbbb bbb refs/heads/main ccc\n`;
    expect(pushedMainCommits(stdin)).toEqual(["aaa", "bbb"]);
  });

  it("ignore les autres branches, les étiquettes et les suppressions", () => {
    const stdin = [
      "refs/heads/x aaa refs/heads/x 000",
      "refs/tags/v1 bbb refs/tags/v1 000",
      `(delete) ${ZERO} refs/heads/main ccc`,
    ].join("\n");
    expect(pushedMainCommits(stdin)).toEqual([]);
  });

  it("rend une liste vide sur une entrée vide", () => {
    expect(pushedMainCommits("")).toEqual([]);
  });
});

describe("readCoverageSummary", () => {
  it("lit le pourcentage de lignes du total", () => {
    const json = JSON.stringify({ total: { lines: { total: 200, covered: 150, pct: 75 } } });
    expect(readCoverageSummary(json)).toEqual({ lines: 75 });
  });

  it("refuse un résumé sans total", () => {
    expect(() => readCoverageSummary("{}")).toThrow("resume de couverture illisible");
  });
});

describe("checkCoverage", () => {
  it("échoue sous le plancher et donne les deux nombres", () => {
    expect(checkCoverage("couverture ts", 61.2, 62)).toEqual({
      ok: false,
      message: "couverture ts : 61.2 % des lignes, sous le plancher de 62 %",
    });
  });

  it("signale un plancher à relever au-delà d'un point d'avance", () => {
    expect(checkCoverage("couverture ts", 64, 62).message).toContain("à relever");
  });

  it("passe au plancher", () => {
    expect(checkCoverage("couverture ts", 62.4, 62).ok).toBe(true);
  });

  it("refuse un plancher absent", () => {
    expect(checkCoverage("couverture ts", 62.4, undefined).ok).toBe(false);
  });
});

describe("combine", () => {
  it("échoue si l'un des contrôles échoue et garde les deux messages", () => {
    expect(combine({ ok: true, message: "a" }, { ok: false, message: "b" })).toEqual({
      ok: false,
      message: "a ; b",
    });
  });
});

describe("summarizeLlvmCov", () => {
  const json = JSON.stringify({
    data: [
      {
        totals: { lines: { count: 1000, covered: 600, percent: 60 } },
        files: [
          {
            filename: "C:\\synk\\src-tauri\\src\\sources\\withings\\client\\session.rs",
            summary: { lines: { count: 200, covered: 160, percent: 80 } },
          },
          {
            filename: "C:\\synk\\src-tauri\\src\\sources\\withings\\client\\oauth.rs",
            summary: { lines: { count: 100, covered: 50, percent: 50 } },
          },
          {
            filename: "C:\\synk\\src-tauri\\src\\db\\pool.rs",
            summary: { lines: { count: 700, covered: 390, percent: 55.7 } },
          },
        ],
      },
    ],
  });

  it("rend le total et la part d'un dossier, pondérée par les lignes", () => {
    expect(summarizeLlvmCov(json, "src/sources/withings/client/")).toEqual({
      total: 60,
      prefix: 70,
    });
  });

  it("refuse un dossier qui ne correspond à aucun fichier", () => {
    expect(() => summarizeLlvmCov(json, "src/inexistant/")).toThrow("aucun fichier");
  });
});
