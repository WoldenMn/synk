import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { safeGetItem, safeSetItem, isBoolean, isString, isStringArray } from "@/lib/storage";

const PERMANENT_ROUTES = ["/", "/export", "/settings"];

const COLOR_THEMES = ["indigo", "emerald", "rose", "amber", "slate"] as const;
type ColorTheme = (typeof COLOR_THEMES)[number];
type Theme = "dark" | "light";

const isTheme = (v: unknown): v is Theme => v === "dark" || v === "light";
const isColorTheme = (v: unknown): v is ColorTheme =>
  isString(v) && (COLOR_THEMES as readonly string[]).includes(v);

function loadHiddenRoutes(): string[] {
  return safeGetItem("synk-hidden-routes", isStringArray) ?? [];
}

function loadSidebarCollapsed(): boolean {
  return safeGetItem("synk-sidebar-collapsed", isBoolean) ?? false;
}

function loadColorTheme(): ColorTheme {
  return safeGetItem("synk-color-theme", isColorTheme) ?? "indigo";
}

function loadTheme(): Theme {
  return safeGetItem("synk-theme", isTheme) ?? "dark";
}

const SIZE_SYSTEMS = ["eu", "us", "uk"] as const;
type SizeSystem = (typeof SIZE_SYSTEMS)[number];
const isSizeSystem = (v: unknown): v is SizeSystem =>
  isString(v) && (SIZE_SYSTEMS as readonly string[]).includes(v);
function loadSizeSystem(): SizeSystem {
  return safeGetItem("synk-size-system", isSizeSystem) ?? "eu";
}

function applyColorTheme(theme: ColorTheme) {
  const el = document.documentElement;
  COLOR_THEMES.forEach((t) => el.classList.remove(`theme-${t}`));
  el.classList.add(`theme-${theme}`);
}

interface UiState {
  theme: Theme;
  colorTheme: ColorTheme;
  sidebarCollapsed: boolean;
  quickAddOpen: boolean;
  hiddenRoutes: string[];
  sizeSystem: SizeSystem;
  toggleTheme: () => void;
  setColorTheme: (theme: ColorTheme) => void;
  toggleSidebar: () => void;
  setQuickAddOpen: (open: boolean) => void;
  toggleRoute: (route: string) => void;
  setSizeSystem: (system: SizeSystem) => void;
}

export { COLOR_THEMES };
export type { ColorTheme };

export const useUiStore = create<UiState>()(
  devtools(
    (set) => ({
      theme: loadTheme(),
      colorTheme: loadColorTheme(),
      sidebarCollapsed: loadSidebarCollapsed(),
      quickAddOpen: false,
      hiddenRoutes: loadHiddenRoutes(),
      sizeSystem: loadSizeSystem(),

      toggleTheme: () =>
        set(
          (state) => {
            const next: Theme = state.theme === "dark" ? "light" : "dark";
            safeSetItem("synk-theme", next);
            if (next === "dark") {
              document.documentElement.classList.add("dark");
            } else {
              document.documentElement.classList.remove("dark");
            }
            return { theme: next };
          },
          false,
          "ui/toggleTheme"
        ),

      setColorTheme: (ct) =>
        set(
          () => {
            safeSetItem("synk-color-theme", ct);
            applyColorTheme(ct);
            return { colorTheme: ct };
          },
          false,
          "ui/setColorTheme"
        ),

      setSizeSystem: (system) =>
        set(
          () => {
            safeSetItem("synk-size-system", system);
            return { sizeSystem: system };
          },
          false,
          "ui/setSizeSystem"
        ),

      toggleSidebar: () =>
        set(
          (state) => {
            const next = !state.sidebarCollapsed;
            safeSetItem("synk-sidebar-collapsed", next);
            return { sidebarCollapsed: next };
          },
          false,
          "ui/toggleSidebar"
        ),

      setQuickAddOpen: (open) => set({ quickAddOpen: open }, false, "ui/setQuickAddOpen"),

      toggleRoute: (route) =>
        set(
          (state) => {
            if (PERMANENT_ROUTES.includes(route)) return state;
            const hidden = state.hiddenRoutes.includes(route)
              ? state.hiddenRoutes.filter((r) => r !== route)
              : [...state.hiddenRoutes, route];
            safeSetItem("synk-hidden-routes", hidden);
            return { hiddenRoutes: hidden };
          },
          false,
          "ui/toggleRoute"
        ),
    }),
    { name: "uiStore", enabled: import.meta.env.DEV }
  )
);
