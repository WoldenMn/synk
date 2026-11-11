import { useCallback, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { SYNC_INVALIDATED_KEYS } from "@/hooks/useSync";
import { reportError } from "@/lib/reportError";
import { logError } from "@/lib/logger";
import { cancelImport } from "./api";
import { IMPORT_PROGRESS_EVENT } from "./events";
import { importToastText } from "./importSummary";
import type { ImportProgress, ImportReport } from "./types";

export type ImportState =
  | { phase: "idle" }
  | { phase: "running"; percent: number; records: number; cancelling: boolean }
  | { phase: "done"; report: ImportReport };

export function useImportRunner(context: string) {
  const queryClient = useQueryClient();
  const [state, setState] = useState<ImportState>({ phase: "idle" });

  const run = useCallback(
    async (execute: () => Promise<ImportReport>): Promise<ImportReport | null> => {
      let off: (() => void) | null = null;
      try {
        setState({ phase: "running", percent: 0, records: 0, cancelling: false });
        off = await listen<ImportProgress>(IMPORT_PROGRESS_EVENT, (event) => {
          setState((s) =>
            s.phase === "running"
              ? { ...s, percent: event.payload.percent, records: event.payload.records }
              : s
          );
        });
        const report = await execute();
        if (report.cancelled) {
          setState({ phase: "idle" });
          toast.info("Import annulé : rien n'a été écrit.");
          return null;
        }
        setState({ phase: "done", report });
        toast.success(importToastText(report));
        for (const queryKey of SYNC_INVALIDATED_KEYS) {
          queryClient.invalidateQueries({ queryKey: [...queryKey] });
        }
        return report;
      } catch (e) {
        setState({ phase: "idle" });
        reportError(e, { source: "manual", context });
        return null;
      } finally {
        off?.();
      }
    },
    [queryClient, context]
  );

  const cancel = useCallback(async () => {
    setState((s) => (s.phase === "running" ? { ...s, cancelling: true } : s));
    try {
      await cancelImport();
    } catch (e) {
      logError("cancel_import failed:", e);
    }
  }, []);

  const reset = useCallback(() => setState({ phase: "idle" }), []);

  return { state, run, cancel, reset };
}
