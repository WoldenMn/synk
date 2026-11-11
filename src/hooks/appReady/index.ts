import { useCallback, useRef, useState } from "react";
import { logInfo, logError } from "@/lib/logger";
import type {
  AppReadyState,
  BootErrorPayload,
  BootSnapshot,
  IntegrityWarningPayload,
} from "./types";
import { useBootListeners } from "./useBootListeners";
import { useBootSnapshot } from "./useBootSnapshot";
import { useBootWatchdog } from "./useBootWatchdog";

export type {
  AppReadyState,
  BootErrorPayload,
  BootSnapshot,
  IntegrityWarningPayload,
} from "./types";
export { STUCK_TIMEOUT_MS } from "./types";

export function useAppReady(): AppReadyState {
  const [state, setState] = useState<AppReadyState>({ status: "loading" });
  const startedAtRef = useRef<number>(Date.now());
  const pendingWarningRef = useRef<IntegrityWarningPayload | undefined>(undefined);

  const handleReady = useCallback((warning?: IntegrityWarningPayload) => {
    const elapsed = Date.now() - startedAtRef.current;
    setState((prev) => {
      if (prev.status === "ready") return prev;
      if (prev.status === "error") return prev;
      logInfo(`[useAppReady] -> ready (after ${elapsed}ms)`);
      return { status: "ready", integrityWarning: warning ?? pendingWarningRef.current };
    });
  }, []);

  const handleError = useCallback((error: BootErrorPayload) => {
    const elapsed = Date.now() - startedAtRef.current;
    setState((prev) => {
      if (prev.status === "ready") return prev;
      if (prev.status === "error") return prev;
      logError(`[useAppReady] -> error (after ${elapsed}ms):`, error);
      return { status: "error", error };
    });
  }, []);

  const handleIntegrity = useCallback((warning: IntegrityWarningPayload) => {
    pendingWarningRef.current = warning;
    setState((prev) =>
      prev.status === "ready" ? { status: "ready", integrityWarning: warning } : prev
    );
  }, []);

  const handleSnapshot = useCallback(
    (snap: BootSnapshot) => {
      if (snap.status === "ready") {
        handleReady(snap.integrity_warning ?? undefined);
      } else if (snap.status === "failed") {
        handleError(snap.error);
      }
    },
    [handleReady, handleError]
  );

  const handleStuck = useCallback((lastSnapshot: BootSnapshot | null, lastError: string | null) => {
    setState((prev) =>
      prev.status === "loading" ? { status: "stuck", lastSnapshot, lastError } : prev
    );
  }, []);

  const { attached } = useBootListeners({
    onReady: handleReady,
    onError: handleError,
    onIntegrity: handleIntegrity,
  });

  useBootSnapshot(attached, handleSnapshot);

  useBootWatchdog(state, {
    onSnapshot: handleSnapshot,
    onStuck: handleStuck,
  });

  return state;
}
