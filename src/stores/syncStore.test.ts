import { describe, it, expect, beforeEach } from "vitest";
import { useSyncStore } from "./syncStore";

describe("syncStore", () => {
  beforeEach(() => {
    localStorage.clear();
    const initial = useSyncStore.getInitialState?.() ?? useSyncStore.getState();
    useSyncStore.setState(initial, true);
  });

  it("expose un état initial avec isSyncing=false et syncProgress vide", () => {
    const state = useSyncStore.getState();
    expect(state.isSyncing).toBe(false);
    expect(state.syncProgress).toBe("");
  });

  it("syncInterval initial vaut 6 par defaut", () => {
    expect(useSyncStore.getState().syncInterval).toBe(6);
  });

  it("setSyncing met a jour isSyncing", () => {
    useSyncStore.getState().setSyncing(true);
    expect(useSyncStore.getState().isSyncing).toBe(true);

    useSyncStore.getState().setSyncing(false);
    expect(useSyncStore.getState().isSyncing).toBe(false);
  });

  it("setSyncProgress met a jour le message", () => {
    useSyncStore.getState().setSyncProgress("Synchronisation des mesures...");
    expect(useSyncStore.getState().syncProgress).toBe("Synchronisation des mesures...");
  });

  it("setSyncInterval met a jour et persiste dans localStorage", () => {
    useSyncStore.getState().setSyncInterval(12);

    expect(useSyncStore.getState().syncInterval).toBe(12);
    expect(localStorage.getItem("synk-sync-interval")).toBe("12");
  });

  it("setSyncInterval accepte 0 (auto-sync off)", () => {
    useSyncStore.getState().setSyncInterval(0);

    expect(useSyncStore.getState().syncInterval).toBe(0);
    expect(localStorage.getItem("synk-sync-interval")).toBe("0");
  });
});
