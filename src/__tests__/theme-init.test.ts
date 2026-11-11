import { afterEach, describe, expect, it } from "vitest";
import tauriConfRaw from "../../src-tauri/tauri.conf.json?raw";
import indexHtml from "../../index.html?raw";
import script from "../../public/theme-init.js?raw";

afterEach(() => {
  document.documentElement.className = "";
  localStorage.clear();
});

describe("thème avant React", () => {
  it("pose le mode et l'accent mémorisés", () => {
    localStorage.setItem("synk-theme", "dark");
    localStorage.setItem("synk-color-theme", "rose");
    new Function(script)();
    expect(document.documentElement.classList.contains("dark")).toBe(true);
    expect(document.documentElement.classList.contains("theme-rose")).toBe(true);
  });

  it("garde le mode clair mémorisé", () => {
    localStorage.setItem("synk-theme", "light");
    new Function(script)();
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });

  it("part en sombre par défaut, comme le store", () => {
    new Function(script)();
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });

  it("ignore un accent inconnu", () => {
    localStorage.setItem("synk-color-theme", "fuchsia");
    new Function(script)();
    expect([...document.documentElement.classList].some((c) => c.startsWith("theme-"))).toBe(false);
  });

  it("lit aussi une valeur écrite entre guillemets", () => {
    localStorage.setItem("synk-theme", '"light"');
    localStorage.setItem("synk-color-theme", '"amber"');
    new Function(script)();
    expect(document.documentElement.classList.contains("dark")).toBe(false);
    expect(document.documentElement.classList.contains("theme-amber")).toBe(true);
  });

  it("tient si le stockage est illisible", () => {
    const original = Storage.prototype.getItem;
    Storage.prototype.getItem = () => {
      throw new Error("bloqué");
    };
    try {
      new Function(script)();
    } finally {
      Storage.prototype.getItem = original;
    }
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });

  it("peint la fenêtre en sombre avant le premier rendu", () => {
    expect(JSON.parse(tauriConfRaw).app.windows[0].backgroundColor).toBe("#080c17");
  });

  it("charge le script comme fichier de l'application, la CSP refusant le script en ligne", () => {
    expect(indexHtml).toContain('<script src="/theme-init.js"></script>');
    expect(JSON.parse(tauriConfRaw).app.security.csp).toContain("script-src 'self'");
  });
});
