import { execSync, spawn } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { connect } from "./parcours-cdp.mjs";
import {
  AXE_PREPARE,
  AXE_PROBE,
  axeScopes,
  axeProblems,
  blindProblem,
  captureName,
  capturePlan,
  FOCUS_PROBE,
  isBlankCapture,
  launchTarget,
  openViewScript,
  pageProblems,
  quickAddProblems,
  ROUTES,
  STYLE_PROBES,
  themeProblem,
  themeScript,
} from "./parcours-lib.mjs";

const PORT = 9333;
const env = {
  ...process.env,
  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT} --remote-allow-origins=*`,
  SYNK_CONTENEUR: "rester",
};
const capturesArg = process.argv.slice(2).find((a) => a.startsWith("--captures="));
const capturesDir = capturesArg ? capturesArg.slice("--captures=".length) : null;
const target = launchTarget(process.argv.slice(2));
const app = spawn(target.command, target.args, { env, shell: target.shell, stdio: "ignore" });

function stopApp() {
  try {
    execSync(`taskkill /pid ${app.pid} /T /F`, { stdio: "ignore" });
  } catch (e) {
    console.error(`[parcours] arret de l'app impossible : ${e.message}`);
  }
}

async function waitForPage() {
  let lastError = "aucune reponse";
  for (let i = 0; i < 600; i++) {
    try {
      const targets = await (await fetch(`http://127.0.0.1:${PORT}/json/list`)).json();
      const page = targets.find((t) => t.type === "page" && t.url.startsWith(target.origin));
      if (page) return page;
      lastError = `cibles : ${targets.map((t) => t.url).join(", ")}`;
    } catch (e) {
      lastError = e.message;
    }
    await sleep(1000);
  }
  throw new Error(`page introuvable sur le port ${PORT} apres 10 min (${lastError})`);
}

const CHECK = `(() => ({
  path: location.pathname,
  title: document.querySelector("header h1")?.textContent?.trim() ?? "",
  docTitle: document.title,
  headingFocused: document.activeElement === document.querySelector("header h1"),
  mainText: (document.querySelector("main")?.innerText ?? "").trim().length,
  boundary: document.body.innerText.includes("Impossible d'afficher cette page"),
}))()`;

let problems = [];
try {
  const page = await waitForPage();
  const cdp = connect(page.webSocketDebuggerUrl);
  await cdp.ready;
  await cdp.send("Runtime.enable");
  await cdp.send("Log.enable");
  const onboardingValue = JSON.stringify(JSON.stringify("true"));
  for (let attempt = 1; ; attempt++) {
    try {
      await cdp.evaluate(`localStorage.setItem("synk-onboarding-done", ${onboardingValue})`);
      break;
    } catch (e) {
      if (attempt >= 5) throw e;
      await sleep(3000);
    }
  }
  await cdp.send("Page.reload");
  await sleep(4000);
  for (let i = 0; i < 60; i++) {
    if (await cdp.evaluate(`Boolean(document.querySelector("header h1"))`)) break;
    await sleep(1000);
  }
  cdp.errors.length = 0;
  const goTo = (route) =>
    cdp.evaluate(
      `history.pushState({}, "", "${route}"); dispatchEvent(new PopStateEvent("popstate"));`
    );
  for (const [index, route] of ROUTES.entries()) {
    await cdp.evaluate(
      `history.pushState({}, "", "${route}"); dispatchEvent(new PopStateEvent("popstate"));`
    );
    await sleep(1500);
    const check = await cdp.evaluate(CHECK);
    const found = pageProblems(
      { ...check, path: route, expectFocus: index > 0 },
      cdp.errors.splice(0)
    );
    console.log(`${found.length === 0 ? "ok    " : "ECHEC "} ${route}`);
    problems = problems.concat(found);
  }
  await cdp.evaluate("document.activeElement?.blur()");
  await goTo("/");
  await sleep(1500);
  const press = async (key, code, text) => {
    for (const type of ["keyDown", "keyUp"]) {
      await cdp.send("Input.dispatchKeyEvent", {
        type,
        key,
        code: key,
        windowsVirtualKeyCode: code,
        ...(type === "keyDown" && text ? { text } : {}),
      });
    }
    await sleep(400);
  };
  await cdp.evaluate(`document.querySelector('button[aria-label="Ajout rapide"]').focus()`);
  await press("Enter", 13, "\r");
  const opened = await cdp.evaluate(FOCUS_PROBE);
  await press("ArrowDown", 40);
  const second = await cdp.evaluate(FOCUS_PROBE);
  await press("Escape", 27);
  const closed = await cdp.evaluate(FOCUS_PROBE);
  const keyboard = quickAddProblems({ opened, second, closed });
  console.log(`${keyboard.length === 0 ? "ok    " : "ECHEC "} ajout rapide au clavier`);
  problems = problems.concat(keyboard);
  await cdp.evaluate(readFileSync("node_modules/axe-core/axe.min.js", "utf8"));
  await cdp.evaluate(AXE_PREPARE);
  mkdirSync("logs", { recursive: true });
  const csvPath = resolve("logs/parcours-balance.csv");
  writeFileSync(csvPath, "Date;Poids (kg)\n25/10/2026;72,4\n26/10/2026;72,1\n");
  const ceilings = JSON.parse(readFileSync("scripts/axe-contraste.json", "utf8")).plafonds;
  const contrastCounts = {};
  for (const scope of axeScopes(ROUTES)) {
    const { mode, route, view, label } = scope;
    await cdp.evaluate(themeScript(mode, scope.accent));
    await goTo(route);
    await sleep(1500);
    if (view && !(await cdp.evaluate(openViewScript(view, csvPath)))) {
      problems.push(`${label} (${mode}) : onglet "${view}" introuvable`);
      continue;
    }
    if (view) await sleep(800);
    const { violations, contrast: seen } = await cdp.evaluate(AXE_PROBE);
    const { problems: found, slack } = axeProblems(label, mode, violations, ceilings);
    const blind = blindProblem(label, mode, seen);
    if (blind) found.push(blind);
    const contrast = violations.find((v) => v.id === "color-contrast");
    contrastCounts[`${mode} ${label}`] = {
      defauts: contrast ? contrast.nodes.length : 0,
      juges: seen.judged,
      nonJuges: seen.incomplete,
      texteDeGraphique: seen.chartText,
    };
    console.log(`${found.length === 0 ? "ok    " : "ECHEC "} axe ${mode} ${label}`);
    for (const s of slack) console.log(`[parcours] plafond de contraste a abaisser : ${s}`);
    if (found.length > 0) {
      for (const v of violations) console.log(`       ${v.id} : ${v.nodes.join(" | ")}`);
    }
    problems = problems.concat(found);
  }
  mkdirSync("logs", { recursive: true });
  writeFileSync("logs/parcours-axe.json", JSON.stringify(contrastCounts, null, 2));
  await cdp.evaluate(themeScript("dark", "indigo"));
  if (capturesDir) {
    mkdirSync(capturesDir, { recursive: true });
    await cdp.send("Emulation.setDeviceMetricsOverride", {
      width: 1920,
      height: 1080,
      deviceScaleFactor: 1,
      mobile: false,
    });
    const manifest = [];
    for (const shot of capturePlan(ROUTES)) {
      await cdp.evaluate(
        `history.pushState({}, "", "${shot.route}"); dispatchEvent(new PopStateEvent("popstate"));`
      );
      await cdp.evaluate(themeScript(shot.mode, shot.accent));
      await cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: 2, y: 2 });
      await sleep(1200);
      await cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: 2, y: 2 });
      await sleep(400);
      const className = await cdp.evaluate("document.documentElement.className");
      const wrong = themeProblem(className, shot.mode, shot.accent);
      if (wrong) problems.push(`${shot.route} : ${wrong}`);
      const file = captureName(shot.route, shot.mode, shot.accent);
      let bytes = Buffer.alloc(0);
      for (let attempt = 0; attempt < 4; attempt++) {
        const png = (await cdp.send("Page.captureScreenshot", { format: "png" })).result.data;
        bytes = Buffer.from(png, "base64");
        if (!isBlankCapture(bytes.length)) break;
        await sleep(2000);
      }
      if (isBlankCapture(bytes.length)) problems.push(`${shot.route} : capture noire (${file})`);
      writeFileSync(join(capturesDir, file), bytes);
      const title = await cdp.evaluate(
        `document.querySelector("header h1")?.textContent?.trim() ?? ""`
      );
      const probes = shot.accent === "indigo" ? await cdp.evaluate(STYLE_PROBES) : null;
      manifest.push({ ...shot, file, className, title, probes });
    }
    writeFileSync(join(capturesDir, "manifest.json"), JSON.stringify(manifest, null, 2));
    for (const e of cdp.errors.splice(0)) problems.push(`captures : console : ${e}`);
  }
  for (const w of cdp.warnings) problems.push(`demarrage ou navigation : ${w}`);
  cdp.close();
} catch (e) {
  problems.push(`parcours interrompu : ${e.message}`);
} finally {
  stopApp();
}

for (const p of problems) console.error(`[parcours] ${p}`);
console.log(`[parcours] ${ROUTES.length} pages, ${problems.length} probleme(s)`);
process.exit(problems.length === 0 ? 0 : 1);
