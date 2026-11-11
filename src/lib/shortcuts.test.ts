import { describe, expect, it } from "vitest";
import { SHORTCUTS } from "./shortcuts";

const WEBVIEW2_ACCELERATORS = [
  "ctrl+r",
  "ctrl+shift+r",
  "f5",
  "ctrl+f5",
  "ctrl+p",
  "ctrl+f",
  "ctrl+g",
  "ctrl+shift+g",
  "ctrl+n",
  "ctrl+shift+n",
  "ctrl+t",
  "ctrl+shift+t",
  "ctrl+w",
  "ctrl+o",
  "ctrl+s",
  "ctrl+d",
  "ctrl+h",
  "ctrl+j",
  "ctrl+l",
  "ctrl+e",
  "ctrl+u",
  "ctrl+shift+i",
  "ctrl+shift+j",
  "ctrl+shift+c",
  "ctrl+shift+delete",
  "f12",
  "f7",
  "f3",
  "ctrl+0",
  "ctrl+plus",
  "ctrl+minus",
  "ctrl+=",
  "ctrl+-",
];

const APP_KEYS = ["escape", "enter", " ", "tab", "arrowup", "arrowdown", "arrowleft", "arrowright"];

function combo(s: { key: string; ctrl: boolean; shift: boolean }): string {
  return [s.ctrl ? "ctrl" : "", s.shift ? "shift" : "", s.key].filter(Boolean).join("+");
}

describe("raccourcis clavier de Synk", () => {
  const combos = Object.values(SHORTCUTS).map(combo);

  it("sont trois combinaisons distinctes", () => {
    expect(combos).toEqual(["ctrl+shift+s", "ctrl+shift+a", "ctrl+b"]);
    expect(new Set(combos).size).toBe(3);
  });

  it("n'entrent en conflit avec aucun accélérateur de WebView2", () => {
    expect(combos.filter((c) => WEBVIEW2_ACCELERATORS.includes(c))).toEqual([]);
  });

  it("n'entrent en conflit avec aucune touche déjà posée dans l'application", () => {
    expect(combos.filter((c) => APP_KEYS.includes(c.split("+").pop() ?? ""))).toEqual([]);
  });

  it("annoncent leur combinaison de façon lisible et pour les lecteurs d'écran", () => {
    for (const s of Object.values(SHORTCUTS)) {
      expect(s.label).toMatch(/^Ctrl\+(Maj\+)?[A-Z]$/);
      expect(s.keyshortcuts).toMatch(/^Control\+(Shift\+)?[A-Z]$/);
    }
  });
});
