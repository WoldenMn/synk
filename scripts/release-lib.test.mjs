import { describe, expect, it } from "vitest";
import {
  bumpAll,
  changelogProblem,
  compareSemver,
  installerProblems,
  isCdpSilence,
  isTransientParcours,
  journalLine,
  parseArgs,
  parseShaLine,
  previousInstaller,
  shaLine,
  stepPlan,
} from "./release-lib.mjs";

describe("parseArgs", () => {
  it("lit la version et le mode", () => {
    expect(parseArgs(["2.23.0"])).toEqual({ version: "2.23.0", simulation: false });
    expect(parseArgs(["2.23.0", "--simulation"])).toEqual({ version: "2.23.0", simulation: true });
    expect(parseArgs(["--simulation", "2.23.0"])).toEqual({ version: "2.23.0", simulation: true });
  });

  it("refuse l'absence de version, une version mal formee et une option inconnue", () => {
    expect(parseArgs([]).error).toMatch(/usage/);
    expect(parseArgs(["2.23"]).error).toMatch(/mal formee/);
    expect(parseArgs(["2.23.0", "--vite"]).error).toMatch(/option inconnue/);
  });
});

describe("compareSemver", () => {
  it("compare numeriquement, pas en texte", () => {
    expect(compareSemver("2.10.0", "2.9.0")).toBe(1);
    expect(compareSemver("2.22.0", "2.22.0")).toBe(0);
    expect(compareSemver("2.22.0", "3.0.0")).toBe(-1);
  });
});

describe("changelogProblem", () => {
  const text = "# Changelog\n\n## [Non publié]\n\n## [2.22.0] - 2026-10-26\n\n### Ajouté\n";

  it("accepte une section datee", () => {
    expect(changelogProblem(text, "2.22.0")).toBeNull();
  });

  it("refuse une version absente ou une section sans date", () => {
    expect(changelogProblem(text, "2.23.0")).toMatch(/2\.23\.0/);
    expect(changelogProblem("## [2.23.0]\n", "2.23.0")).toMatch(/date/);
  });

  it("refuse un Non publie qui contient encore des notes", () => {
    const pending = "## [Non publié]\n\n- une note\n\n## [2.23.0] - 2026-10-27\n";
    expect(changelogProblem(pending, "2.23.0")).toMatch(/Non publi/);
  });
});

describe("somme de l'installeur", () => {
  it("ecrit le format des releases deja archivees", () => {
    const line = shaLine(
      "57b6e8afc5b73d3e574cf511cd0cd94434f14d48fec6bab23b6ac0c3d970e3a87".slice(0, 64),
      "Synk_2.22.0_x64-setup.exe"
    );
    expect(line).toMatch(/^[0-9A-F]{64} {2}Synk_2\.22\.0_x64-setup\.exe\n$/);
  });

  it("relit ce qu'elle a ecrit, et tolere les fins de ligne Windows", () => {
    const hash = "A".repeat(64);
    const line = shaLine(hash, "Synk_2.22.0_x64-setup.exe");
    expect(parseShaLine(line)).toEqual({ hash, name: "Synk_2.22.0_x64-setup.exe" });
    expect(parseShaLine(line.replace("\n", "\r\n"))).toEqual({
      hash,
      name: "Synk_2.22.0_x64-setup.exe",
    });
    expect(parseShaLine("pas une somme")).toBeNull();
  });
});

describe("previousInstaller", () => {
  const names = [
    "Synk_2.9.0_x64-setup.exe",
    "Synk_2.21.0_x64-setup.exe",
    "Synk_2.21.0_x64-setup.exe.sha256",
    "Synk_2.22.0_x64-setup.exe",
    "journal-des-releases.txt",
  ];

  it("rend la version immediatement inferieure, comparee en nombres", () => {
    expect(previousInstaller(names, "2.23.0")).toBe("Synk_2.22.0_x64-setup.exe");
    expect(previousInstaller(names, "2.22.0")).toBe("Synk_2.21.0_x64-setup.exe");
    expect(previousInstaller(names, "2.9.0")).toBeNull();
  });
});

describe("installerProblems", () => {
  const good = {
    name: "Synk_2.23.0_x64-setup.exe",
    size: 7_000_000,
    mtimeMs: 2_000,
    buildStartMs: 1_000,
    fileVersion: "2.23.0",
    version: "2.23.0",
    previousSize: 6_942_445,
  };

  it("ne dit rien d'un installeur sain", () => {
    expect(installerProblems(good)).toEqual([]);
  });

  it("refuse un installeur plus vieux que le build", () => {
    expect(installerProblems({ ...good, mtimeMs: 500 }).join()).toMatch(/anterieur au build/);
  });

  it("refuse une version de fichier qui n'est pas celle visee", () => {
    expect(installerProblems({ ...good, fileVersion: "2.22.0" }).join()).toMatch(/2\.22\.0/);
  });

  it("refuse un fichier vide ou minuscule", () => {
    expect(installerProblems({ ...good, size: 0 }).join()).toMatch(/taille/);
    expect(installerProblems({ ...good, size: 900_000 }).join()).toMatch(/taille/);
  });

  it("refuse une hausse de poids de plus de 10 % sur la version precedente (N-53)", () => {
    expect(installerProblems({ ...good, size: 7_700_000 }).join()).toMatch(/hausse/);
    expect(installerProblems({ ...good, size: 7_600_000 })).toEqual([]);
  });

  it("n'a rien a comparer a la premiere release", () => {
    expect(installerProblems({ ...good, previousSize: null })).toEqual([]);
  });
});

describe("isCdpSilence", () => {
  it("reconnait le silence du navigateur integre, rien d'autre", () => {
    expect(
      isCdpSilence(
        "[parcours] parcours interrompu : aucune reponse CDP a Runtime.evaluate apres 30000 ms"
      )
    ).toBe(true);
    expect(isCdpSilence("[parcours] /sleep : titre absent")).toBe(false);
  });
});

describe("isTransientParcours", () => {
  const ipc =
    "[parcours] demarrage ou navigation : IPC replie sur postMessage (CSP ?) : IPC custom protocol failed\n    at invoke (x)";
  const pages = "ok     /\nok     /body\n";

  it("accepte le silence CDP et le seul repli de l'IPC au demarrage", () => {
    expect(isTransientParcours("[parcours] parcours interrompu : aucune reponse CDP")).toBe(true);
    expect(isTransientParcours(`${pages}[parcours] 19 pages, 2 probleme(s)\n${ipc}\n${ipc}`)).toBe(
      true
    );
  });

  it("refuse une page en echec, meme avec le repli de l'IPC", () => {
    expect(
      isTransientParcours(
        `${pages}ECHEC  /sleep\n[parcours] 19 pages, 2 probleme(s)\n${ipc}\n[parcours] /sleep : titre absent`
      )
    ).toBe(false);
    expect(isTransientParcours(`${pages}[parcours] /sleep : titre absent`)).toBe(false);
  });
});

describe("stepPlan", () => {
  const ids = (plan) => plan.map((s) => s.id);

  it("enchaine veille, version, verification, mesures, build, copie, commit", () => {
    expect(ids(stepPlan({ simulation: false }))).toEqual([
      "controles",
      "veille",
      "version",
      "verify",
      "couverture",
      "parcours",
      "budget",
      "build",
      "installeur",
      "copie",
    ]);
  });

  it("la simulation ne fait ni build, ni copie, ni commit", () => {
    const plan = ids(stepPlan({ simulation: true }));
    expect(plan).not.toContain("build");
    expect(plan).not.toContain("copie");
    expect(plan).not.toContain("installeur");
    expect(plan).toContain("veille");
    expect(plan).toContain("verify");
    expect(plan).toContain("parcours");
  });
});

describe("bumpAll", () => {
  const files = {
    "package.json": '{\n  "version": "2.22.0"\n}\n',
    "package-lock.json":
      '{\n  "version": "2.22.0",\n  "packages": {\n    "": {\n      "version": "2.22.0"\n    }\n  }\n}\n',
    "src-tauri/Cargo.toml": '[package]\nname = "synk"\nversion = "2.22.0"\n',
    "src-tauri/Cargo.lock": '[[package]]\nname = "synk"\nversion = "2.22.0"\n',
    "src-tauri/tauri.conf.json": '{\n  "version": "2.22.0"\n}\n',
    "README.md": "Version courante : 2.22.0 (x).\n",
  };

  it("rend les textes montes sans rien ecrire et sans emplacement oublie", () => {
    const { texts, wrong } = bumpAll((p) => files[p], "2.23.0");
    expect(wrong).toEqual({});
    expect(texts["package.json"]).toContain("2.23.0");
    expect(files["package.json"]).toContain("2.22.0");
  });

  it("refuse de partir de cinq fichiers en desaccord", () => {
    const mixed = { ...files, "package.json": '{\n  "version": "2.21.0"\n}\n' };
    expect(() => bumpAll((p) => mixed[p], "2.23.0")).toThrow(/desaccord/);
  });
});

describe("journalLine", () => {
  it("porte la version, le mode, chaque etape en secondes, la memoire et la somme", () => {
    const line = journalLine({
      date: "2026-10-26T12:00:00.000Z",
      version: "2.23.0",
      simulation: false,
      steps: [
        { id: "verify", seconds: 240 },
        { id: "build", seconds: 300 },
      ],
      minFreeMb: 4100,
      sha: "AB".repeat(32),
      bytes: 7_000_000,
      tree: "abc123",
    });
    expect(line).toContain("2.23.0");
    expect(line).toContain("verify=240s");
    expect(line).toContain("build=300s");
    expect(line).toContain("memoire_libre_min=4100Mo");
    expect(line).toContain("arbre=abc123");
    expect(line).toContain("sha256=" + "AB".repeat(32));
    expect(line.endsWith("\n")).toBe(true);
  });

  it("marque la simulation", () => {
    const line = journalLine({
      date: "d",
      version: "2.23.0",
      simulation: true,
      steps: [],
      minFreeMb: 1,
      sha: null,
      bytes: null,
      tree: "t",
    });
    expect(line).toContain("simulation");
    expect(line).not.toContain("sha256=");
  });
});
