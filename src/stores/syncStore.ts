import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { safeGetItem, safeSetItem, isString } from "@/lib/storage";

interface SyncState {
  isSyncing: boolean;
  syncProgress: string;
  syncInterval: number;
  setSyncing: (syncing: boolean) => void;
  setSyncProgress: (progress: string) => void;
  setSyncInterval: (hours: number) => void;
}

const initialInterval = (() => {
  const raw = safeGetItem("synk-sync-interval", isString);
  const n = Number(raw ?? "6");
  return Number.isFinite(n) && n >= 0 ? n : 6;
})();

export const useSyncStore = create<SyncState>()(
  devtools(
    (set) => ({
      isSyncing: false,
      syncProgress: "",
      syncInterval: initialInterval,

      setSyncing: (syncing) => set({ isSyncing: syncing }, false, "sync/setSyncing"),

      setSyncProgress: (progress) => set({ syncProgress: progress }, false, "sync/setSyncProgress"),

      setSyncInterval: (hours) => {
        safeSetItem("synk-sync-interval", String(hours));
        set({ syncInterval: hours }, false, "sync/setSyncInterval");
      },
    }),
    { name: "syncStore", enabled: import.meta.env.DEV }
  )
);
