import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { logInfo, logError } from "@/lib/logger";
import type { BootErrorPayload, IntegrityWarningPayload } from "./types";

export function useBootListeners(callbacks: {
  onReady: () => void;
  onError: (error: BootErrorPayload) => void;
  onIntegrity: (warning: IntegrityWarningPayload) => void;
}): { attached: boolean } {
  const { onReady, onError, onIntegrity } = callbacks;
  const [attached, setAttached] = useState(false);

  useEffect(() => {
    let cancelled = false;
    let unlistenReady: (() => void) | null = null;
    let unlistenError: (() => void) | null = null;
    let unlistenIntegrity: (() => void) | null = null;
    logInfo("[useAppReady] mount, attaching listeners");

    Promise.all([
      listen("app-ready", () => onReady()),
      listen<BootErrorPayload>("app-error", (event) => onError(event.payload)),
      listen<IntegrityWarningPayload>("app-integrity-warning", (event) =>
        onIntegrity(event.payload)
      ),
    ])
      .then(([fnReady, fnError, fnIntegrity]) => {
        if (cancelled) {
          fnReady();
          fnError();
          fnIntegrity();
          return;
        }
        unlistenReady = fnReady;
        unlistenError = fnError;
        unlistenIntegrity = fnIntegrity;
        setAttached(true);
      })
      .catch((e) => {
        logError("[useAppReady] listen() failed:", e);
      });

    return () => {
      cancelled = true;
      unlistenReady?.();
      unlistenError?.();
      unlistenIntegrity?.();
    };
  }, [onReady, onError, onIntegrity]);

  return { attached };
}
