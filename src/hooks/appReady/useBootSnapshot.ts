import { useEffect, useRef } from "react";
import { getBootState } from "./api";
import { logWarn } from "@/lib/logger";
import type { BootSnapshot } from "./types";

export function useBootSnapshot(enabled: boolean, onSnapshot: (snap: BootSnapshot) => void): void {
  const calledRef = useRef(false);

  useEffect(() => {
    if (!enabled || calledRef.current) return;
    calledRef.current = true;
    let cancelled = false;

    getBootState()
      .then((snap) => {
        if (cancelled) return;
        onSnapshot(snap);
      })
      .catch((e) => {
        logWarn("[useAppReady] get_boot_state failed (waiting for event):", e);
      });

    return () => {
      cancelled = true;
    };
  }, [enabled, onSnapshot]);
}
