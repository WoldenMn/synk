import { useCallback, useState } from "react";
import { toast } from "sonner";
import { listen } from "@tauri-apps/api/event";
import { useQueryClient } from "@tanstack/react-query";
import { useSyncStore } from "@/stores/syncStore";
import { DASHBOARD_CARDS_KEY } from "@/features/dashboard/hooks";
import { syncAllSources, syncSource } from "@/features/sources/api";
import { SOURCES_KEY } from "@/features/sources/hooks";
import { SYNC_PROGRESS_EVENT } from "@/features/sources/events";
import { sourceLabel } from "@/features/sources/labels";
import type { SourceOutcome, SyncProgress } from "@/features/sources/types";
import { reportError } from "@/lib/reportError";

export const SYNC_INVALIDATED_KEYS: readonly (readonly string[])[] = [
  ["sync", "last-success"],
  SOURCES_KEY,
  ["bodyMeasures"],
  ["bloodPressure"],
  ["sleep"],
  ["activity"],
  ["workouts"],
  ["heart"],
  ["devices"],
  ["insights"],
  ["sync_log"],
  ["metric-series"],
  ["metric-latest"],
  ["metric-definitions"],
  [DASHBOARD_CARDS_KEY],
];

function plural(n: number, word: string): string {
  return `${n} ${word}${n !== 1 ? "s" : ""}`;
}

function announce(outcomes: SourceOutcome[]) {
  if (outcomes.length === 0) {
    toast.info("Aucune source connectée : connectez-en une dans Paramètres.");
    return;
  }
  const total = outcomes.reduce((sum, o) => sum + o.count, 0);
  const errors = outcomes.flatMap((o) => o.errors);
  if (errors.length === 0) {
    toast.success(
      `Synchronisation terminée : ${plural(total, "enregistrement")} importé${total !== 1 ? "s" : ""}`
    );
    return;
  }
  toast.warning(
    `Sync partielle : ${plural(total, "enregistrement")}. Erreurs : ${errors.join(", ")}`,
    { duration: 10000 }
  );
  for (const outcome of outcomes) {
    for (const message of outcome.errors) {
      reportError(message, {
        source: "sync",
        context: `sync_source:${outcome.source}`,
        toast: false,
      });
    }
  }
}

export function useSync(sourceId?: string) {
  const isSyncing = useSyncStore((s) => s.isSyncing);
  const syncProgress = useSyncStore((s) => s.syncProgress);
  const setSyncing = useSyncStore((s) => s.setSyncing);
  const setSyncProgress = useSyncStore((s) => s.setSyncProgress);
  const queryClient = useQueryClient();
  const [syncingHere, setSyncingHere] = useState(false);

  const sync = useCallback(async () => {
    if (useSyncStore.getState().isSyncing) return;
    setSyncing(true);
    setSyncingHere(true);
    setSyncProgress("Démarrage...");
    let unlisten: (() => void) | null = null;
    try {
      unlisten = await listen<SyncProgress>(SYNC_PROGRESS_EVENT, (event) => {
        const { source, libelle, etape, total } = event.payload;
        setSyncProgress(`${sourceLabel(source)} : ${libelle} (${etape}/${total})`);
      });
      const outcomes = sourceId ? await syncSource(sourceId) : await syncAllSources();
      for (const queryKey of SYNC_INVALIDATED_KEYS) {
        queryClient.invalidateQueries({ queryKey: [...queryKey] });
      }
      announce(outcomes);
    } catch (e) {
      reportError(e, {
        source: "sync",
        context: sourceId ? `sync_source:${sourceId}` : "sync_all_sources",
        prefix: "Échec de la synchronisation : ",
      });
    } finally {
      unlisten?.();
      setSyncing(false);
      setSyncingHere(false);
      setSyncProgress("");
    }
  }, [setSyncing, setSyncProgress, queryClient, sourceId]);

  return { sync, isSyncing, syncProgress, syncingHere };
}
