import { lazy, Suspense, Component, useEffect } from "react";
import type { ErrorInfo, ReactNode } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClientProvider } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { Toaster } from "sonner";
import { AppLayout } from "@/components/layout/AppLayout";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";
import { LoadingSkeleton } from "@/components/shared";
import { BootErrorScreen } from "@/components/shared/BootErrorScreen";
import { BootStuckScreen } from "@/components/shared/BootStuckScreen";
import { useUiStore } from "@/stores/uiStore";
import { useAppReady } from "@/hooks/useAppReady";
import { prefetchDashboard } from "@/hooks/appReady/bootPrefetch";
import { logError } from "@/lib/logger";
import { reportError } from "@/lib/reportError";
import { createQueryClient } from "@/lib/queryClient";

const Dashboard = lazy(() => import("@/pages/Dashboard"));
const BodyComposition = lazy(() => import("@/pages/BodyComposition"));
const BloodPressure = lazy(() => import("@/pages/BloodPressure"));
const Sleep = lazy(() => import("@/pages/Sleep"));
const Activity = lazy(() => import("@/pages/Activity"));
const Workouts = lazy(() => import("@/pages/Workouts"));
const Heart = lazy(() => import("@/pages/Heart"));
const Measurements = lazy(() => import("@/pages/Measurements"));
const Alerts = lazy(() => import("@/pages/Alerts"));
const Profile = lazy(() => import("@/pages/Profile"));
const Analyses = lazy(() => import("@/pages/Analyses"));
const Medications = lazy(() => import("@/pages/Medications"));
const Labs = lazy(() => import("@/pages/Labs"));
const Insights = lazy(() => import("@/pages/Insights"));
const Explorer = lazy(() => import("@/pages/Explorer"));
const Journal = lazy(() => import("@/pages/Journal"));
const Timeline = lazy(() => import("@/pages/Timeline"));
const Export = lazy(() => import("@/pages/Export"));
const Settings = lazy(() => import("@/pages/Settings"));

const queryClient = createQueryClient();

class ErrorBoundary extends Component<
  { children: ReactNode },
  { hasError: boolean; error: string }
> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { hasError: false, error: "" };
  }
  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error: error.message };
  }
  componentDidCatch(error: Error, info: ErrorInfo) {
    logError("Synk error:", error, info);
    reportError(error, { source: "boundary", context: "react", toast: false });
  }
  render() {
    if (this.state.hasError) {
      return (
        <div className="flex h-screen items-center justify-center bg-background p-8">
          <div className="text-center space-y-4 max-w-md">
            <h1 className="text-2xl font-bold text-destructive-text">Erreur</h1>
            <p className="text-muted-foreground">{this.state.error}</p>
            <button
              onClick={() => {
                this.setState({ hasError: false, error: "" });
                window.location.reload();
              }}
              className="inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
            >
              Recharger
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

function PageFallback() {
  return (
    <div className="space-y-6">
      <LoadingSkeleton rows={3} height="h-20" />
      <LoadingSkeleton rows={6} />
    </div>
  );
}

export default function App() {
  const appState = useAppReady();
  const theme = useUiStore((s) => s.theme);

  useEffect(() => {
    let cancelled = false;
    const unlistenPromise = listen("app-ready", async () => {
      if (cancelled) return;
      await prefetchDashboard(queryClient);
    });
    return () => {
      cancelled = true;
      unlistenPromise.then((fn) => fn());
    };
  }, []);

  if (appState.status === "error") {
    return <BootErrorScreen error={appState.error} />;
  }

  if (appState.status === "stuck") {
    return <BootStuckScreen lastSnapshot={appState.lastSnapshot} lastError={appState.lastError} />;
  }

  if (appState.status === "loading") {
    return (
      <div className="flex h-screen items-center justify-center bg-background">
        <div className="flex flex-col items-center space-y-4">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
          <p className="text-sm text-muted-foreground">Chargement de Synk...</p>
        </div>
      </div>
    );
  }

  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <ConfirmProvider>
          <BrowserRouter>
            <Suspense fallback={<PageFallback />}>
              <Routes>
                <Route element={<AppLayout integrityWarning={appState.integrityWarning} />}>
                  <Route path="/" element={<Dashboard />} />
                  <Route path="/body" element={<BodyComposition />} />
                  <Route path="/blood-pressure" element={<BloodPressure />} />
                  <Route path="/sleep" element={<Sleep />} />
                  <Route path="/activity" element={<Activity />} />
                  <Route path="/workouts" element={<Workouts />} />
                  <Route path="/heart" element={<Heart />} />
                  <Route path="/measurements" element={<Measurements />} />
                  <Route path="/alerts" element={<Alerts />} />
                  <Route path="/profile" element={<Profile />} />
                  <Route path="/analyses" element={<Analyses />} />
                  <Route path="/medications" element={<Medications />} />
                  <Route path="/labs" element={<Labs />} />
                  <Route path="/insights" element={<Insights />} />
                  <Route path="/explorer" element={<Explorer />} />
                  <Route path="/journal" element={<Journal />} />
                  <Route path="/timeline" element={<Timeline />} />
                  <Route path="/export" element={<Export />} />
                  <Route path="/settings" element={<Settings />} />
                </Route>
              </Routes>
            </Suspense>
          </BrowserRouter>
        </ConfirmProvider>
        <Toaster richColors position="bottom-right" closeButton theme={theme} />
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
