export const ROUTES = [
  "/",
  "/body",
  "/blood-pressure",
  "/sleep",
  "/activity",
  "/workouts",
  "/heart",
  "/measurements",
  "/alerts",
  "/profile",
  "/analyses",
  "/medications",
  "/labs",
  "/insights",
  "/explorer",
  "/journal",
  "/timeline",
  "/export",
  "/settings",
];

export const IPC_FALLBACK = "IPC custom protocol failed";

export function warningProblem(text) {
  return text.includes(IPC_FALLBACK) ? `IPC replie sur postMessage (CSP ?) : ${text}` : null;
}

export function launchTarget(argv) {
  const exe = argv.find((a) => a.startsWith("--exe="));
  return exe
    ? {
        command: exe.slice("--exe=".length),
        args: [],
        shell: false,
        origin: "http://tauri.localhost",
      }
    : { command: "npm", args: ["run", "tauri:dev"], shell: true, origin: "http://localhost:1420" };
}

export function routesFromApp(source) {
  return [...source.matchAll(/<Route\s+path="([^"]+)"/g)].map((m) => m[1]);
}

export function pageProblems(check, errors) {
  const problems = [];
  if (!check.title) problems.push(`${check.path} : titre absent`);
  if (check.mainText === 0) problems.push(`${check.path} : zone principale vide`);
  if (check.boundary) problems.push(`${check.path} : écran d'erreur de route`);
  if (check.docTitle !== undefined && !check.docTitle.endsWith(" | Synk")) {
    problems.push(`${check.path} : titre de fenêtre "${check.docTitle}"`);
  }
  if (check.expectFocus && !check.headingFocused) {
    problems.push(`${check.path} : le focus n'est pas sur le titre de la page`);
  }
  for (const e of errors) problems.push(`${check.path} : console : ${e}`);
  return problems;
}

export const CONTRAST_RULE = "color-contrast";

export const AXE_NEUTRALISE_CSS =
  ".bg-gradient-to-br { background-image: none !important; background-color: var(--color-background) !important; }";

export const AXE_PREPARE = `(() => {
  if (document.getElementById("axe-neutralise")) return;
  const style = document.createElement("style");
  style.id = "axe-neutralise";
  style.textContent = ${JSON.stringify(AXE_NEUTRALISE_CSS)};
  document.head.append(style);
})()`;

export const AXE_PROBE = `(async () => {
  const r = await axe.run(document, { resultTypes: ["violations", "incomplete", "passes"] });
  const nodes = (list) => list.filter((v) => v.id === "color-contrast").flatMap((v) => v.nodes);
  const count = (list) => nodes(list).length;
  const chartText = (n) => n.any.some((c) => c.data?.messageKey === "imgNode");
  return {
    violations: r.violations.map((v) => ({
      id: v.id,
      impact: v.impact,
      nodes: v.nodes.map((n) => n.target.join(" ")),
    })),
    contrast: {
      judged: count(r.passes) + count(r.violations),
      incomplete: nodes(r.incomplete).filter((n) => !chartText(n)).length,
      chartText: nodes(r.incomplete).filter(chartText).length,
    },
  };
})()`;

export const AXE_ACCENTS = ["indigo", "rose"];

export const CSV_VIEW = "Assistant CSV";

export const AXE_VIEWS = [
  { route: "/measurements", view: "Tailles" },
  { route: "/settings", view: "Priorités" },
  { route: "/settings", view: CSV_VIEW },
];

export function axeScopes(routes, accents = AXE_ACCENTS, views = AXE_VIEWS) {
  const scopes = [];
  for (const accent of accents) {
    for (const mode of ["light", "dark"]) {
      const suffix = accent === "indigo" ? [] : [accent];
      for (const route of routes) {
        scopes.push({
          mode,
          accent,
          route,
          view: null,
          label: suffix.length ? `${route} [${suffix.join(", ")}]` : route,
        });
      }
      for (const { route, view } of views) {
        scopes.push({
          mode,
          accent,
          route,
          view,
          label: `${route} [${[view, ...suffix].join(", ")}]`,
        });
      }
    }
  }
  return scopes;
}

export function viewScript(view) {
  return `(() => {
    const button = [...document.querySelectorAll("button")].find((b) => b.textContent.trim() === ${JSON.stringify(view)});
    if (!button) return false;
    button.click();
    return true;
  })()`;
}

export function csvAssistantScript(path) {
  return `(async () => {
    if (!window.__parcoursDialogStub) {
      window.__parcoursDialogStub = true;
      const real = window.fetch;
      window.fetch = (input, init) => {
        const url = String(input && input.url ? input.url : input);
        if (url.includes("plugin%3Adialog%7Copen")) {
          return Promise.resolve(
            new Response(JSON.stringify(${JSON.stringify(path)}), {
              status: 200,
              headers: { "Tauri-Response": "ok", "Content-Type": "application/json" },
            })
          );
        }
        return real.call(window, input, init);
      };
    }
    const home = [...document.querySelectorAll('[role="tab"]')].find((t) => t.textContent.trim() === "Sources");
    if (home) {
      home.click();
      await new Promise((resolve) => setTimeout(resolve, 300));
    }
    const button = [...document.querySelectorAll("button")].find(
      (b) => b.textContent.includes("Choisir un fichier") && b.textContent.includes("Fichiers CSV")
    );
    if (!button) return false;
    button.click();
    for (let i = 0; i < 40; i++) {
      if (document.querySelector('[aria-label="Assistant de correspondance CSV"]')) return true;
      await new Promise((resolve) => setTimeout(resolve, 200));
    }
    return false;
  })()`;
}

export function openViewScript(view, csvPath) {
  return view === CSV_VIEW ? csvAssistantScript(csvPath) : viewScript(view);
}

export const BLIND_SHARE = 0.25;

export function blindProblem(route, mode, contrast) {
  const total = contrast.judged + contrast.incomplete;
  if (total === 0 || contrast.incomplete / total <= BLIND_SHARE) return null;
  return `${route} (${mode}) : axe ne juge pas le contraste de ${contrast.incomplete} élément(s) sur ${total}`;
}

export function axeProblems(route, mode, violations, ceilings = {}) {
  const problems = [];
  const key = `${mode} ${route}`;
  let contrast = 0;
  for (const v of violations) {
    if (v.id === CONTRAST_RULE) contrast += v.nodes.length;
    else {
      problems.push(
        `${route} (${mode}) : axe ${v.id} (${v.impact ?? "impact inconnu"}) sur ${v.nodes.length} noeud(s)`
      );
    }
  }
  const ceiling = ceilings[key] ?? 0;
  const slack = [];
  if (contrast > ceiling) {
    problems.push(
      `${route} (${mode}) : contraste insuffisant sur ${contrast} noeud(s), plafond ${ceiling}`
    );
  } else if (contrast < ceiling) {
    slack.push(`${key} : ${contrast} noeud(s) pour un plafond de ${ceiling}, à abaisser`);
  }
  return { problems, slack };
}

export const FOCUS_PROBE = `(() => {
  const a = document.activeElement;
  return {
    role: a?.getAttribute("role") ?? "",
    label: a?.getAttribute("aria-label") ?? a?.textContent?.trim() ?? "",
    menu: Boolean(document.querySelector('[role="menu"]')),
  };
})()`;

export function quickAddProblems({ opened, second, closed }) {
  const problems = [];
  if (opened.role !== "menuitem" || !opened.menu) {
    problems.push(
      "ajout rapide : Entrée sur le bouton n'ouvre pas le menu avec le focus sur une option"
    );
  }
  if (second.role !== "menuitem" || second.label === opened.label) {
    problems.push("ajout rapide : la flèche bas ne déplace pas le focus sur l'option suivante");
  }
  if (closed.menu || !closed.label.startsWith("Ajout rapide")) {
    problems.push("ajout rapide : Échap ne ferme pas le menu ni ne rend le focus au bouton");
  }
  return problems;
}

export const ACCENTS = ["indigo", "emerald", "rose", "amber", "slate"];

export function captureName(route, mode, accent) {
  const slug = route === "/" ? "accueil" : route.replace(/^\//, "").replaceAll("/", "-");
  return `${mode}-${accent}-${slug}.png`;
}

export function capturePlan(routes) {
  const plan = [];
  for (const route of routes)
    for (const mode of ["light", "dark"]) plan.push({ route, mode, accent: "indigo" });
  for (const accent of ACCENTS.filter((a) => a !== "indigo"))
    for (const mode of ["light", "dark"]) plan.push({ route: "/", mode, accent });
  return plan;
}

export function themeScript(mode, accent) {
  return `(() => {
    const el = document.documentElement;
    [...el.classList].filter((c) => c === "dark" || c.startsWith("theme-")).forEach((c) => el.classList.remove(c));
    if (${JSON.stringify(mode)} === "dark") el.classList.add("dark");
    el.classList.add(${JSON.stringify(`theme-${accent}`)});
  })()`;
}

export function themeProblem(className, mode, accent) {
  const classes = className.split(/\s+/);
  const modeOk = mode === "dark" ? classes.includes("dark") : !classes.includes("dark");
  return modeOk && classes.includes(`theme-${accent}`)
    ? null
    : `theme non applique : attendu ${mode}/${accent}, classes "${className}"`;
}

export const STYLE_PROBES = `(() => {
  const pick = (sel, prop) => {
    const el = document.querySelector(sel);
    return el ? getComputedStyle(el)[prop] : "absent";
  };
  const input = document.querySelector("input[placeholder]");
  return {
    rounded: pick(".rounded", "borderTopLeftRadius"),
    roundedLg: pick(".rounded-lg", "borderTopLeftRadius"),
    borderColor: pick(".border", "borderTopColor"),
    buttonCursor: pick("button:not(:disabled)", "cursor"),
    placeholder: input ? getComputedStyle(input, "::placeholder").color : "absent",
    focusOutline: pick(".outline-none, .outline-hidden", "outlineStyle"),
  };
})()`;

export const MIN_CAPTURE_BYTES = 30000;

export function isBlankCapture(byteLength) {
  return byteLength < MIN_CAPTURE_BYTES;
}
