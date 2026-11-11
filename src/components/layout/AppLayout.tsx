import { lazy, Suspense, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { useQueryClient } from "@tanstack/react-query";
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";
import { QuickAddButton } from "./QuickAddButton";
import { RouteErrorBoundary } from "./RouteErrorBoundary";
import { IntegrityBanner } from "@/components/shared/IntegrityBanner";
import type { IntegrityWarningPayload } from "@/hooks/appReady/types";
import { useUiStore } from "@/stores/uiStore";
import { useSyncStore } from "@/stores/syncStore";
import { useSync } from "@/hooks/useSync";
import { toast } from "sonner";
import { notifyError } from "@/lib/reportError";
import { getLastSuccessfulSync } from "@/features/syncLog/api";
import { getBackupFailure } from "@/features/backups/api";
import { getInvalidatedSecrets, listSources } from "@/features/sources/api";
import { BACKUP_FAILED_EVENT } from "@/features/sources/events";
import { SOURCES_KEY, useSourceEvents, useSources } from "@/features/sources/hooks";
import {
  isReconnectRequired,
  reconnectReason,
  reconnectSteps,
  reconnectTitle,
} from "@/features/sources/reconnectNotice";
import { showsQuickAdd } from "@/lib/quickAdd";
import { useLayoutShortcuts } from "@/hooks/useLayoutShortcuts";
import { readOnboardingDone } from "@/lib/onboarding";
import { errorDetail } from "@/lib/errors";
import { logWarn } from "@/lib/logger";
import { scheduleIdle } from "@/lib/scheduleIdle";
import { useRouteAnnouncement } from "@/hooks/useRouteAnnouncement";

const OnboardingGate = lazy(() =>
  import("@/components/shared/OnboardingWizard/OnboardingGate").then((m) => ({
    default: m.OnboardingGate,
  }))
);

export function AppLayout({ integrityWarning }: { integrityWarning?: IntegrityWarningPayload }) {
  const theme = useUiStore((s) => s.theme);
  const colorTheme = useUiStore((s) => s.colorTheme);
  const queryClient = useQueryClient();
  const { sync } = useSync();
  const autoSyncRan = useRef(false);
  const mainRef = useRef<HTMLElement>(null);
  const headingRef = useRef<HTMLHeadingElement>(null);
  useRouteAnnouncement(mainRef, headingRef);
  useLayoutShortcuts(sync);
  const { pathname } = useLocation();
  const [showOnboarding, setShowOnboarding] = useState(() => !readOnboardingDone());
  const closeOnboarding = useCallback(() => setShowOnboarding(false), []);

  useEffect(() => {
    if (theme === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [theme]);

  useEffect(() => {
    const el = document.documentElement;
    ["indigo", "emerald", "rose", "amber", "slate"].forEach((t) =>
      el.classList.remove(`theme-${t}`)
    );
    el.classList.add(`theme-${colorTheme}`);
  }, [colorTheme]);

  useEffect(() => {
    if (autoSyncRan.current) return;
    autoSyncRan.current = true;
    let cancelled = false;
    let started = false;

    // Annule le rappel au demontage : sans cela, un AppLayout demonte avant
    // son rappel lancait quand meme le controle et son toast.
    const cancelIdle = scheduleIdle(() => {
      started = true;
      (async () => {
        try {
          const sources = await queryClient.fetchQuery({
            queryKey: SOURCES_KEY,
            queryFn: listSources,
            staleTime: 60 * 1000,
          });
          if (cancelled) return;
          const invalidated = await getInvalidatedSecrets();
          if (cancelled) return;
          const states = Object.fromEntries(
            sources.map((s) => [
              s.id,
              { connected: s.connected, credentialsConfigured: s.credentials_configured },
            ])
          );
          if (isReconnectRequired(invalidated) && reconnectSteps(invalidated, states).length > 0) {
            toast.warning(
              `${reconnectTitle(invalidated)}. ${reconnectReason(invalidated)} Ouvrez Paramètres pour vous reconnecter.`,
              { duration: 20000 }
            );
          }
          if (!sources.some((s) => s.connected)) return;

          const interval = useSyncStore.getState().syncInterval;
          if (interval === 0) return;

          const lastSync = await getLastSuccessfulSync();
          if (cancelled) return;
          if (lastSync) {
            const elapsed = Date.now() - new Date(lastSync).getTime();
            const thresholdMs = interval * 60 * 60 * 1000;
            if (elapsed < thresholdMs) return;
          }

          sync();
        } catch (e) {
          logWarn("[AppLayout] auto-sync échoué (non bloquant)", e);
        }
      })();
    });

    return () => {
      cancelled = true;
      cancelIdle();
      // Demonte avant le rappel (double montage de StrictMode) : le montage
      // suivant doit pouvoir le reprogrammer.
      if (!started) autoSyncRan.current = false;
    };
  }, [sync, queryClient]);

  const { data: sources } = useSources();
  const sourceIds = useMemo(() => (sources ?? []).map((s) => s.id), [sources]);
  const onAuthSuccess = useCallback(() => {
    queryClient.invalidateQueries({ queryKey: SOURCES_KEY });
    sync();
  }, [queryClient, sync]);
  useSourceEvents("auth-success", sourceIds, onAuthSuccess);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | null = null;
    const notify = (failure: unknown) =>
      notifyError(`Sauvegarde quotidienne impossible : ${errorDetail(failure)}`, "backup-failed");
    listen<unknown>(BACKUP_FAILED_EVENT, (event) => notify(event.payload)).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    getBackupFailure()
      .then((failure) => {
        if (failure && !cancelled) notify(failure);
      })
      .catch((e) => logWarn("[AppLayout] état de la sauvegarde illisible", e));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return (
    <>
      {showOnboarding && (
        <Suspense fallback={null}>
          <OnboardingGate onDone={closeOnboarding} />
        </Suspense>
      )}
      <div className="flex h-screen overflow-hidden bg-gradient-to-br from-background via-background to-primary/5">
        <a
          href="#main-content"
          className="sr-only focus:not-sr-only focus:absolute focus:z-50 focus:p-4 focus:bg-background focus:text-foreground focus:rounded-md"
        >
          Aller au contenu
        </a>
        <Sidebar />
        <div className="flex flex-1 flex-col overflow-hidden">
          <Header headingRef={headingRef} />
          <main ref={mainRef} id="main-content" className="flex-1 overflow-y-auto p-6">
            <IntegrityBanner warning={integrityWarning} />
            <RouteErrorBoundary>
              <Outlet />
            </RouteErrorBoundary>
          </main>
        </div>
        {showsQuickAdd(pathname) && <QuickAddButton />}
      </div>
    </>
  );
}
