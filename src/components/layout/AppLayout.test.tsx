import { beforeAll, beforeEach, describe, it, expect, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createQueryClient } from "@/lib/queryClient";
import { useUiStore } from "@/stores/uiStore";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve(false)),
}));
vi.mock("@tauri-apps/api/event", () => ({
  // listen() retourne une fonction unlisten — vi.fn() est appelable et
  // suffit comme cleanup no-op pour les tests.
  listen: vi.fn(() => Promise.resolve(vi.fn())),
}));
vi.mock("sonner", () => ({
  toast: { warning: vi.fn(), success: vi.fn(), error: vi.fn() },
}));
vi.mock("@/lib/db", () => ({
  updateUserProfile: vi.fn(),
  getUserProfile: vi.fn(() => Promise.resolve(null)),
  getMeasurementTypes: vi.fn(() => Promise.resolve([])),
  getGoals: vi.fn(() => Promise.resolve([])),
  getWithingsDevices: vi.fn(() => Promise.resolve([])),
}));

import { AppLayout } from "./AppLayout";
import { defined } from "@/test-utils/defined";

function renderLayout() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={client}>
      <MemoryRouter>
        <Routes>
          <Route path="/" element={<AppLayout />}>
            <Route index element={<div>Home</div>} />
          </Route>
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("AppLayout skip link", () => {
  it("renders a skip link targeting #main-content", () => {
    renderLayout();
    const skip = screen.getByText(/aller au contenu/i);
    expect(skip).toBeInTheDocument();
    expect(skip.tagName).toBe("A");
    expect(skip.getAttribute("href")).toBe("#main-content");
  });

  it("renders main with id=main-content", () => {
    renderLayout();
    const main = document.getElementById("main-content");
    expect(main?.tagName).toBe("MAIN");
  });
});

function sourceInfo(id: string, label: string, connected: boolean) {
  return {
    id,
    label,
    kind: "oauth",
    domains: [],
    connected,
    credentials_configured: connected,
    last_sync_at: null,
    redirect_uri: null,
    developer_url: null,
  };
}

function mockBackend(
  invalidated: {
    sources: { source: string; label: string; tokens: boolean; credentials: boolean }[];
  },
  sources = [sourceInfo("withings", "Withings", false)]
) {
  vi.mocked(invoke).mockImplementation((cmd: string) => {
    if (cmd === "list_sources") return Promise.resolve(sources);
    if (cmd === "get_invalidated_secrets") return Promise.resolve(invalidated);
    return Promise.resolve(null);
  });
}

describe("AppLayout reconnexion", () => {
  it("previent au démarrage quand des secrets Withings ont été invalides", async () => {
    vi.mocked(toast.warning).mockClear();
    mockBackend({
      sources: [{ source: "withings", label: "Withings", tokens: true, credentials: true }],
    });

    renderLayout();

    await waitFor(() => expect(toast.warning).toHaveBeenCalledTimes(1));
    expect(defined(vi.mocked(toast.warning).mock.calls[0])[0]).toMatch(
      /Reconnexion Withings nécessaire/
    );
  });

  it("previent pour une autre source même quand Withings est connecté (N-62)", async () => {
    vi.mocked(toast.warning).mockClear();
    mockBackend(
      { sources: [{ source: "oura", label: "Oura", tokens: true, credentials: false }] },
      [sourceInfo("withings", "Withings", true), sourceInfo("oura", "Oura", false)]
    );

    renderLayout();

    await waitFor(() => expect(toast.warning).toHaveBeenCalledTimes(1));
    expect(defined(vi.mocked(toast.warning).mock.calls[0])[0]).toMatch(
      /Reconnexion Oura nécessaire/
    );
  });

  it("reste silencieux quand aucun secret n'a été invalide", async () => {
    vi.mocked(toast.warning).mockClear();
    mockBackend({ sources: [] });

    renderLayout();

    await waitFor(() => expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_invalidated_secrets"));
    expect(toast.warning).not.toHaveBeenCalled();
  });
});

describe("AppLayout connexion d'une source", () => {
  it("lance une synchronisation dès withings-auth-success", async () => {
    const handlers = new Map<string, (event: { payload: unknown }) => void>();
    vi.mocked(listen).mockImplementation(((
      event: string,
      handler: (event: { payload: unknown }) => void
    ) => {
      handlers.set(event, handler);
      return Promise.resolve(vi.fn());
    }) as unknown as typeof listen);
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "sync_all_sources") return Promise.resolve([]);
      if (cmd === "list_sources")
        return Promise.resolve([sourceInfo("withings", "Withings", false)]);
      return Promise.resolve(null);
    });

    renderLayout();
    await waitFor(() => expect(handlers.has("withings-auth-success")).toBe(true));
    await act(async () => {
      handlers.get("withings-auth-success")?.({ payload: null });
    });

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("sync_all_sources"));
  });
});

describe("AppLayout sauvegarde quotidienne", () => {
  it("signale une sauvegarde quotidienne impossible", async () => {
    const handlers = new Map<string, (event: { payload: unknown }) => void>();
    vi.mocked(listen).mockImplementation(((
      event: string,
      handler: (event: { payload: unknown }) => void
    ) => {
      handlers.set(event, handler);
      return Promise.resolve(vi.fn());
    }) as unknown as typeof listen);
    vi.mocked(toast.error).mockClear();

    renderLayout();
    await waitFor(() => expect(handlers.has("backup-failed")).toBe(true));
    handlers.get("backup-failed")?.({
      payload: {
        kind: "Internal",
        message: "Erreur interne : disque plein",
        detail: "disque plein",
      },
    });

    expect(toast.error).toHaveBeenCalledWith(
      "Sauvegarde quotidienne impossible : disque plein",
      expect.objectContaining({ duration: 10_000, id: "backup-failed" })
    );
  });

  it("signale un échec survenu avant l'écoute de l'évènement", async () => {
    vi.mocked(listen).mockImplementation((() =>
      Promise.resolve(vi.fn())) as unknown as typeof listen);
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      Promise.resolve(
        cmd === "get_backup_failure"
          ? { kind: "Internal", message: "Erreur interne : disque plein", detail: "disque plein" }
          : null
      )
    );
    vi.mocked(toast.error).mockClear();

    renderLayout();

    await waitFor(() =>
      expect(toast.error).toHaveBeenCalledWith(
        "Sauvegarde quotidienne impossible : disque plein",
        expect.objectContaining({ id: "backup-failed" })
      )
    );
  });
});

describe("AppLayout lectures de l'en-tête", () => {
  it("garde l'en-tête quand la dernière synchro est illisible", async () => {
    vi.mocked(listen).mockImplementation((() =>
      Promise.resolve(vi.fn())) as unknown as typeof listen);
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      cmd === "get_last_successful_sync"
        ? Promise.reject("Erreur base de donnees : verrou")
        : Promise.resolve(null)
    );
    render(
      <QueryClientProvider client={createQueryClient()}>
        <MemoryRouter>
          <Routes>
            <Route path="/" element={<AppLayout />}>
              <Route index element={<div>Accueil</div>} />
            </Route>
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>
    );
    expect(
      await screen.findByText("Dernière synchro inconnue", {}, { timeout: 4000 })
    ).toBeInTheDocument();
  });
});

describe("AppLayout contrôle d'intégrité", () => {
  it("affiche le bandeau quand la base a échoué au contrôle", () => {
    render(
      <QueryClientProvider client={createQueryClient()}>
        <MemoryRouter>
          <Routes>
            <Route
              path="/"
              element={<AppLayout integrityWarning={{ Details: "page 3 corrompue" }} />}
            >
              <Route index element={<div>Accueil</div>} />
            </Route>
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>
    );
    expect(screen.getByText("page 3 corrompue")).toBeInTheDocument();
  });
});

function installation(
  counts: Partial<Record<"body_measures_count" | "sleep_count", number>>,
  sources = [sourceInfo("withings", "Withings", false)]
) {
  vi.mocked(invoke).mockImplementation((cmd: string) => {
    if (cmd === "get_db_metrics")
      return Promise.resolve({
        db_size_bytes: 4096,
        audit_log_count: 0,
        body_measures_count: 0,
        blood_pressure_count: 0,
        sleep_count: 0,
        activity_count: 0,
        workouts_count: 0,
        ...counts,
      });
    if (cmd === "list_sources") return Promise.resolve(sources);
    if (cmd === "get_invalidated_secrets") return Promise.resolve({ sources: [] });
    return Promise.resolve(null);
  });
}

describe("AppLayout assistant de premier lancement", () => {
  beforeAll(async () => {
    await import("@/components/shared/OnboardingWizard/OnboardingGate");
  });
  beforeEach(() => localStorage.clear());

  it("s'ouvre sur une installation neuve", async () => {
    installation({});
    renderLayout();
    expect(await screen.findByRole("dialog")).toBeInTheDocument();
  });

  it("reste fermé quand la base a déjà des mesures, et retient qu'il est passé", async () => {
    installation({ body_measures_count: 256 });
    renderLayout();
    await screen.findByText("Home");
    await waitFor(() => expect(localStorage.getItem("synk-onboarding-done")).toBe("true"));
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("reste fermé quand une source est déjà connectée", async () => {
    installation({}, [sourceInfo("withings", "Withings", true)]);
    renderLayout();
    await screen.findByText("Home");
    await waitFor(() => expect(localStorage.getItem("synk-onboarding-done")).toBe("true"));
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("s'ouvre quand l'état de l'installation est illisible", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_db_metrics") return Promise.reject(new Error("base occupée"));
      if (cmd === "list_sources") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    renderLayout();
    expect(await screen.findByRole("dialog")).toBeInTheDocument();
  });

  it("ne se rouvre pas une fois termine, valeur ecrite par l'application", async () => {
    localStorage.setItem("synk-onboarding-done", "true");
    renderLayout();
    await screen.findByText("Home");
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 300));
    });
    expect(screen.queryByRole("dialog")).toBeNull();
  });
});

describe("AppLayout raccourcis clavier", () => {
  beforeEach(() => {
    localStorage.setItem("synk-onboarding-done", "true");
    useUiStore.setState({ sidebarCollapsed: false, quickAddOpen: false });
  });

  it("Ctrl+B replie la barre latérale", async () => {
    renderLayout();
    await screen.findByText("Home");
    fireEvent.keyDown(window, { key: "b", ctrlKey: true });
    expect(useUiStore.getState().sidebarCollapsed).toBe(true);
  });

  it("Ctrl+Maj+A ouvre l'ajout rapide", async () => {
    renderLayout();
    await screen.findByText("Home");
    fireEvent.keyDown(window, { key: "A", ctrlKey: true, shiftKey: true });
    expect(useUiStore.getState().quickAddOpen).toBe(true);
  });

  it("Ctrl+Maj+S lance la synchronisation", async () => {
    vi.mocked(invoke).mockClear();
    mockBackend({ sources: [] });
    renderLayout();
    await screen.findByText("Home");
    fireEvent.keyDown(window, { key: "S", ctrlKey: true, shiftKey: true });
    await waitFor(() =>
      expect(vi.mocked(invoke).mock.calls.map((c) => c[0])).toContain("sync_all_sources")
    );
  });
});
