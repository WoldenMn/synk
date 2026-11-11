import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  axeProblems,
  axeScopes,
  viewScript,
  openViewScript,
  CSV_VIEW,
  blindProblem,
  captureName,
  capturePlan,
  isBlankCapture,
  launchTarget,
  pageProblems,
  quickAddProblems,
  ROUTES,
  routesFromApp,
  themeProblem,
  warningProblem,
} from "./parcours-lib.mjs";

describe("ROUTES", () => {
  it("couvre exactement les routes déclarées dans App.tsx", () => {
    expect([...ROUTES].sort()).toEqual(routesFromApp(readFileSync("src/App.tsx", "utf-8")).sort());
  });

  it("lit bien un fichier de routes (contrôle de l'instrument)", () => {
    const source = `<Route path="/" element={<A />} />\n<Route path="/sleep" element={<B />} />`;
    expect(routesFromApp(source)).toEqual(["/", "/sleep"]);
  });
});

describe("pageProblems", () => {
  const healthy = { path: "/sleep", title: "Sommeil", mainText: 120, boundary: false };

  it("ne dit rien d'une page saine", () => {
    expect(pageProblems(healthy, [])).toEqual([]);
  });

  it("nomme chaque défaut", () => {
    expect(
      pageProblems({ path: "/journal", title: "", mainText: 0, boundary: true }, [
        "TypeError: x is undefined",
      ])
    ).toEqual([
      "/journal : titre absent",
      "/journal : zone principale vide",
      "/journal : écran d'erreur de route",
      "/journal : console : TypeError: x is undefined",
    ]);
  });
});

describe("warningProblem", () => {
  it("signale le repli de l'IPC sur postMessage", () => {
    expect(
      warningProblem(
        "IPC custom protocol failed, Tauri will now use the postMessage interface instead TypeError"
      )
    ).toMatch(/postMessage/);
  });

  it("ignore un avertissement ordinaire", () => {
    expect(warningProblem("React Router Future Flag Warning")).toBeNull();
  });
});

describe("launchTarget", () => {
  it("lance la copie de dev par défaut", () => {
    expect(launchTarget([])).toMatchObject({ command: "npm", origin: "http://localhost:1420" });
  });

  it("lance un exécutable release quand --exe est donné", () => {
    expect(launchTarget(["--exe=src-tauri/target/release/synk.exe"])).toMatchObject({
      command: "src-tauri/target/release/synk.exe",
      shell: false,
      origin: "http://tauri.localhost",
    });
  });
});

describe("capturePlan", () => {
  it("couvre chaque page dans les deux modes, et les cinq accents sur le tableau de bord", () => {
    const plan = capturePlan(["/", "/journal"]);
    expect(plan).toHaveLength(2 * 2 + 4 * 2);
    expect(new Set(plan.map((p) => captureName(p.route, p.mode, p.accent))).size).toBe(plan.length);
  });
});

describe("captureName", () => {
  it("nomme l'accueil et les sous-pages sans barre oblique", () => {
    expect(captureName("/", "light", "indigo")).toBe("light-indigo-accueil.png");
    expect(captureName("/settings/withings", "dark", "rose")).toBe(
      "dark-rose-settings-withings.png"
    );
  });
});

describe("themeProblem", () => {
  it("accepte des classes conformes", () => {
    expect(themeProblem("dark theme-rose", "dark", "rose")).toBeNull();
  });

  it("signale un mode sombre absent", () => {
    expect(themeProblem("theme-rose", "dark", "rose")).toMatch(/non applique/);
  });

  it("signale un accent qui n'a pas pris", () => {
    expect(themeProblem("theme-indigo", "light", "rose")).toMatch(/non applique/);
  });
});

describe("isBlankCapture", () => {
  it("juge noire une capture de 8 513 octets (mesure du 2026-10-23) et saine une de 105 480", () => {
    expect(isBlankCapture(8513)).toBe(true);
    expect(isBlankCapture(105480)).toBe(false);
  });
});

describe("pageProblems : annonce de la page", () => {
  const base = { path: "/sleep", title: "Sommeil", mainText: 120, boundary: false };

  it("accepte une fenêtre nommée et le focus sur le titre", () => {
    expect(
      pageProblems(
        { ...base, docTitle: "Sommeil | Synk", expectFocus: true, headingFocused: true },
        []
      )
    ).toEqual([]);
  });

  it("signale une fenêtre sans nom de page", () => {
    expect(pageProblems({ ...base, docTitle: "Synk" }, [])).toEqual([
      '/sleep : titre de fenêtre "Synk"',
    ]);
  });

  it("signale un focus resté ailleurs après un changement de page", () => {
    expect(
      pageProblems(
        { ...base, docTitle: "Sommeil | Synk", expectFocus: true, headingFocused: false },
        []
      )
    ).toEqual(["/sleep : le focus n'est pas sur le titre de la page"]);
  });
});

describe("axeProblems", () => {
  const contrast = (n) => ({
    id: "color-contrast",
    impact: "serious",
    nodes: Array.from({ length: n }, (_, i) => `#n${i}`),
  });

  it("ne dit rien d'une page sans violation (cas connu bon)", () => {
    expect(axeProblems("/", "light", [])).toEqual({ problems: [], slack: [] });
  });

  it("refuse toute violation de contraste sans plafond (cas connu mauvais)", () => {
    expect(axeProblems("/", "dark", [contrast(3)]).problems).toEqual([
      "/ (dark) : contraste insuffisant sur 3 noeud(s), plafond 0",
    ]);
  });

  it("tolère le contraste jusqu'au plafond et dit quand il peut baisser", () => {
    const ceilings = { "dark /": 3 };
    expect(axeProblems("/", "dark", [contrast(3)], ceilings)).toEqual({ problems: [], slack: [] });
    expect(axeProblems("/", "dark", [contrast(1)], ceilings).slack).toEqual([
      "dark / : 1 noeud(s) pour un plafond de 3, à abaisser",
    ]);
    expect(axeProblems("/", "dark", [], ceilings).slack).toHaveLength(1);
  });

  it("refuse toute autre règle, plafond ou non", () => {
    const heading = { id: "heading-order", impact: "moderate", nodes: ["h3"] };
    expect(axeProblems("/sleep", "light", [heading], { "light /sleep": 9 }).problems).toEqual([
      "/sleep (light) : axe heading-order (moderate) sur 1 noeud(s)",
    ]);
  });
});

describe("quickAddProblems", () => {
  const good = {
    opened: { role: "menuitem", label: "Poids", menu: true },
    second: { role: "menuitem", label: "Tension", menu: true },
    closed: { role: "", label: "Ajout rapide", menu: false },
  };

  it("accepte le parcours clavier attendu", () => {
    expect(quickAddProblems(good)).toEqual([]);
  });

  it("nomme chaque étape qui échoue", () => {
    expect(
      quickAddProblems({
        opened: { role: "", label: "Ajout rapide", menu: false },
        second: { role: "", label: "Ajout rapide", menu: false },
        closed: { role: "menuitem", label: "Poids", menu: true },
      })
    ).toHaveLength(3);
  });
});

describe("blindProblem", () => {
  it("accepte une page dont le contraste est presque entièrement jugé", () => {
    expect(blindProblem("/", "dark", { judged: 40, incomplete: 5 })).toBeNull();
  });

  it("accepte une page sans texte à juger", () => {
    expect(blindProblem("/", "dark", { judged: 0, incomplete: 0 })).toBeNull();
  });

  it("refuse une page que axe ne peut pas juger (mesure du 2026-10-23 : 54 sur 63 sous le dégradé)", () => {
    expect(blindProblem("/", "dark", { judged: 9, incomplete: 54 })).toMatch(/54 élément/);
  });
});

describe("axeScopes", () => {
  it("passe chaque page sous l'accent de référence, sous un second accent, puis les vues à onglets", () => {
    const scopes = axeScopes(["/", "/measurements"]);
    expect(scopes.map((s) => s.label)).toEqual([
      "/",
      "/measurements",
      "/measurements [Tailles]",
      "/settings [Priorités]",
      "/settings [Assistant CSV]",
      "/",
      "/measurements",
      "/measurements [Tailles]",
      "/settings [Priorités]",
      "/settings [Assistant CSV]",
      "/ [rose]",
      "/measurements [rose]",
      "/measurements [Tailles, rose]",
      "/settings [Priorités, rose]",
      "/settings [Assistant CSV, rose]",
      "/ [rose]",
      "/measurements [rose]",
      "/measurements [Tailles, rose]",
      "/settings [Priorités, rose]",
      "/settings [Assistant CSV, rose]",
    ]);
    expect(scopes.map((s) => s.mode).slice(0, 10)).toEqual([
      "light",
      "light",
      "light",
      "light",
      "light",
      "dark",
      "dark",
      "dark",
      "dark",
      "dark",
    ]);
  });

  it("garde les clés de plafond des pages de référence (mode puis page)", () => {
    const [first] = axeScopes(["/sleep"]);
    expect(`${first.mode} ${first.label}`).toBe("light /sleep");
  });

  it("clique l'onglet par son texte exact", () => {
    expect(viewScript("Tailles")).toContain('=== "Tailles"');
    expect(openViewScript("Priorités", "C:/x.csv")).toContain('=== "Priorités"');
  });

  it("ouvre l'assistant CSV en répondant au sélecteur de fichier par un chemin fixe", () => {
    const script = openViewScript(CSV_VIEW, "C:/logs/balance.csv");
    expect(script).toContain('"C:/logs/balance.csv"');
    expect(script).toContain("plugin%3Adialog%7Copen");
    expect(script).toContain("Assistant de correspondance CSV");
    expect(script).not.toContain('=== "Assistant CSV"');
  });

  it("passe l'assistant CSV et l'onglet Priorités sous axe, en clair et en sombre", () => {
    const labels = axeScopes(["/settings"]).map((s) => `${s.mode} ${s.label}`);
    expect(labels).toContain("light /settings [Priorités]");
    expect(labels).toContain("dark /settings [Assistant CSV]");
  });
});
