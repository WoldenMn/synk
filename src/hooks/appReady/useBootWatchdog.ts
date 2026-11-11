import { useEffect } from "react";
import { getBootState } from "./api";
import { logWarn, logError } from "@/lib/logger";
import { errorText } from "@/lib/errors";
import type { AppReadyState, BootSnapshot } from "./types";
import { STUCK_TIMEOUT_MS } from "./types";

export function useBootWatchdog(
  state: AppReadyState,
  callbacks: {
    onSnapshot: (snap: BootSnapshot) => void;
    onStuck: (lastSnapshot: BootSnapshot | null, lastError: string | null) => void;
  }
): void {
  const { onSnapshot, onStuck } = callbacks;

  useEffect(() => {
    if (state.status !== "loading") return;
    let cancelled = false;

    const watchdog = window.setTimeout(() => {
      if (cancelled) return;
      logWarn(`[useAppReady] WATCHDOG: still loading after ${STUCK_TIMEOUT_MS}ms`);
      getBootState()
        .then((snap) => {
          if (cancelled) return;
          logWarn("[useAppReady] WATCHDOG snapshot:", snap);
          if (snap.status === "ready" || snap.status === "failed") {
            onSnapshot(snap);
            return;
          }
          onStuck(snap, null);
        })
        .catch((e) => {
          if (cancelled) return;
          const errStr = errorText(e);
          logError("[useAppReady] WATCHDOG get_boot_state failed:", errStr);
          onStuck(null, errStr);
        });
    }, STUCK_TIMEOUT_MS);

    return () => {
      cancelled = true;
      window.clearTimeout(watchdog);
    };
  }, [state.status, onSnapshot, onStuck]);
}
