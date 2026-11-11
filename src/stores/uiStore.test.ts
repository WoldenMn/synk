import { describe, it, expect, beforeEach, vi } from "vitest";
import { useUiStore } from "./uiStore";

describe("uiStore", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.className = "";
    const initial = useUiStore.getInitialState?.() ?? useUiStore.getState();
    useUiStore.setState(initial, true);
  });

  it("expose un thème initial", () => {
    const theme = useUiStore.getState().theme;
    expect(["light", "dark"]).toContain(theme);
  });

  it("toggleTheme inverse le thème", () => {
    const before = useUiStore.getState().theme;
    useUiStore.getState().toggleTheme();
    const after = useUiStore.getState().theme;
    expect(after).not.toBe(before);
  });

  it("setColorTheme met a jour la palette", () => {
    useUiStore.getState().setColorTheme("emerald");
    expect(useUiStore.getState().colorTheme).toBe("emerald");
  });

  it("sizeSystem par defaut est eu", () => {
    expect(useUiStore.getState().sizeSystem).toBe("eu");
  });

  it("setSizeSystem met a jour l'état et persiste", () => {
    useUiStore.getState().setSizeSystem("us");
    expect(useUiStore.getState().sizeSystem).toBe("us");
    expect(localStorage.getItem("synk-size-system")).toBe("us");
  });

  it("toggleSidebar écrit le choix dans le stockage", () => {
    useUiStore.getState().toggleSidebar();
    expect(useUiStore.getState().sidebarCollapsed).toBe(true);
    expect(localStorage.getItem("synk-sidebar-collapsed")).toBe("true");
    useUiStore.getState().toggleSidebar();
    expect(localStorage.getItem("synk-sidebar-collapsed")).toBe("false");
  });

  it("relit la barre latérale repliée au démarrage suivant", async () => {
    localStorage.setItem("synk-sidebar-collapsed", "true");
    vi.resetModules();
    const fresh = await import("./uiStore");
    expect(fresh.useUiStore.getState().sidebarCollapsed).toBe(true);
  });

  it("ignore une valeur étrangère et démarre dépliée", async () => {
    localStorage.setItem("synk-sidebar-collapsed", '"oui"');
    vi.resetModules();
    const fresh = await import("./uiStore");
    expect(fresh.useUiStore.getState().sidebarCollapsed).toBe(false);
  });
});
