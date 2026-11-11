import { describe, expect, it } from "vitest";
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

const NPM_AUDIT = {
  vulnerabilities: {
    braces: {
      severity: "high",
      via: [
        {
          source: 1,
          name: "braces",
          title: "Regex DoS",
          url: "https://github.com/advisories/GHSA-vfj7-8cjw-p6xm",
          severity: "high",
        },
      ],
    },
    micromatch: { severity: "high", via: ["braces"] },
    "left-pad": {
      severity: "low",
      via: [
        {
          source: 2,
          name: "left-pad",
          title: "bénin",
          url: "https://github.com/advisories/GHSA-aaaa-bbbb-cccc",
          severity: "low",
        },
      ],
    },
  },
};

const CARGO_AUDIT = {
  vulnerabilities: {
    list: [
      {
        advisory: { id: "RUSTSEC-2026-0187", title: "Stack exhaustion" },
        package: { name: "lopdf", version: "0.31.0" },
      },
    ],
  },
  warnings: {
    unmaintained: [
      {
        advisory: { id: "RUSTSEC-2025-0056", title: "adler is unmaintained" },
        package: { name: "adler", version: "1.0.2" },
      },
    ],
  },
};

describe("npmAuditFindings", () => {
  it("garde les avis hauts et critiques, une fois chacun", () => {
    expect(npmAuditFindings(NPM_AUDIT)).toEqual([
      { id: "GHSA-vfj7-8cjw-p6xm", paquet: "braces", severite: "high", titre: "Regex DoS" },
    ]);
  });

  it("rend une liste vide pour un audit propre", () => {
    expect(npmAuditFindings({ vulnerabilities: {} })).toEqual([]);
  });
});

describe("cargoAuditFindings", () => {
  it("rend les vulnérabilités et les avertissements", () => {
    expect(cargoAuditFindings(CARGO_AUDIT).map((f) => `${f.genre}:${f.id}`)).toEqual([
      "vulnerabilite:RUSTSEC-2026-0187",
      "unmaintained:RUSTSEC-2025-0056",
    ]);
  });
});

describe("breakingBehind", () => {
  it("compte la majeure à partir de 1.0", () => {
    expect(breakingBehind("3.6.3", "4.2.0")).toBe(true);
    expect(breakingBehind("2.11.2", "2.11.6")).toBe(false);
  });

  it("compte la mineure en 0.x", () => {
    expect(breakingBehind("0.10", "0.11.1")).toBe(true);
    expect(breakingBehind("0.12.8", "0.12.9")).toBe(false);
  });

  it("lit une exigence Cargo avec opérateur", () => {
    expect(breakingBehind("=0.7", "0.12.8")).toBe(true);
    expect(breakingBehind("^1", "1.40.0")).toBe(false);
  });
});

describe("npmMajorFindings", () => {
  it("rend les paquets en retard d'une ligne cassante", () => {
    const outdated = {
      eslint: { current: "9.39.5", latest: "10.12.0" },
      vite: { current: "8.3.2", latest: "8.3.4" },
    };
    expect(npmMajorFindings(outdated)).toEqual([
      { id: "majeure:eslint", paquet: "eslint", actuelle: "9.39.5", derniere: "10.12.0" },
    ]);
  });
});

describe("msrvProblem", () => {
  it("signale un plancher déclaré sous le maximum de l'arbre", () => {
    expect(msrvProblem("1.77.2", ["1.70", "1.88.0", "1.85"])).toMatch(/1\.88\.0/);
  });

  it("accepte un plancher égal ou supérieur", () => {
    expect(msrvProblem("1.88", ["1.70", "1.88.0"])).toBeNull();
  });
});

describe("applyExceptions", () => {
  const findings = [{ id: "A" }, { id: "B" }, { id: "C" }];
  const exceptions = [
    { id: "A", motif: "inatteignable", date: "2026-10-03", echeance: "2027-01-03" },
    { id: "B", motif: "ancien", date: "2026-06-01", echeance: "2026-09-01" },
  ];

  it("excuse, signale l'échue et laisse échouer le reste", () => {
    const r = applyExceptions(findings, exceptions, "2026-10-03");
    expect(r.excused.map((f) => f.id)).toEqual(["A"]);
    expect(r.expired.map((f) => f.id)).toEqual(["B"]);
    expect(r.failing.map((f) => f.id)).toEqual(["B", "C"]);
  });

  it("refuse une exception sans motif", () => {
    const r = applyExceptions(
      [{ id: "A" }],
      [{ id: "A", motif: "", date: "2026-10-03", echeance: "2027-01-03" }],
      "2026-10-03"
    );
    expect(r.failing.map((f) => f.id)).toEqual(["A"]);
  });
});

const INTENDANCE = {
  genere_le: "2026-10-23T03:48:48+02:00",
  constats: [
    {
      genre: "faille",
      composant: "Synk",
      paquet: "braces",
      ecosysteme: "npm",
      severite: "haute",
      identifiants: ["GHSA-vfj7-8cjw-p6xm", "CVE-2026-93687"],
      titre: "braces vulnerable",
    },
    {
      genre: "faille",
      composant: "Synk",
      paquet: "rsa",
      ecosysteme: "cargo",
      severite: "moyenne",
      identifiants: ["GHSA-4grx-2x9w-596c", "RUSTSEC-2023-0071"],
      titre: "Marvin",
    },
    {
      genre: "faille",
      composant: "Autre",
      paquet: "x",
      ecosysteme: "npm",
      severite: "critique",
      identifiants: ["GHSA-zzzz-zzzz-zzzz"],
      titre: "ailleurs",
    },
    { genre: "mise_a_jour", composant: "Synk", paquet: "react", identifiants: [] },
  ],
};

describe("intendanceFindings", () => {
  it("garde les failles hautes et critiques du projet, sous leur identifiant GHSA ou RUSTSEC", () => {
    expect(intendanceFindings(INTENDANCE, "Synk")).toEqual([
      {
        id: "GHSA-vfj7-8cjw-p6xm",
        paquet: "braces",
        severite: "haute",
        titre: "braces vulnerable",
        alias: ["GHSA-vfj7-8cjw-p6xm", "CVE-2026-93687"],
      },
    ]);
  });

  it("rend une liste vide pour un projet absent", () => {
    expect(intendanceFindings(INTENDANCE, "Inconnu")).toEqual([]);
  });
});

describe("snapshotCovers", () => {
  it("accepte un instantané plus récent que les verrous", () => {
    expect(snapshotCovers("2026-10-23T03:48:48+02:00", "2026-10-22T17:00:00+02:00")).toBe(true);
  });

  it("refuse un instantané antérieur à un verrou modifié", () => {
    expect(snapshotCovers("2026-10-23T03:48:48+02:00", "2026-10-23T10:00:22+02:00")).toBe(false);
  });

  it("refuse un instantané ou une date illisible", () => {
    expect(snapshotCovers("n'importe quoi", "2026-10-23T10:00:22+02:00")).toBe(false);
    expect(snapshotCovers("2026-10-23T03:48:48+02:00", "")).toBe(false);
  });
});
