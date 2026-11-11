import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import React from "react";
import { useSync } from "./useSync";
import { toast } from "sonner";
import { useSyncStore } from "@/stores/syncStore";

vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  },
}));

vi.mock("@/lib/reportError", () => ({
  reportError: vi.fn(),
}));

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function makeWrapper(client: QueryClient) {
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return Wrapper;
}

const outcome = (source: string, count: number, errors: string[] = [], failed = false) => ({
  source,
  label: source === "withings" ? "Withings" : "Oura",
  count,
  errors,
  failed,
});

function freshClient() {
  return new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
}

async function runSync(qc: QueryClient) {
  const { result } = renderHook(() => useSync(), { wrapper: makeWrapper(qc) });
  await act(async () => {
    await result.current.sync();
  });
}

describe("useSync", () => {
  let qc: QueryClient;

  beforeEach(async () => {
    qc = freshClient();
    mockInvoke.mockReset();
    useSyncStore.setState({ isSyncing: false, syncProgress: "" });
    vi.mocked(toast.success).mockClear();
    vi.mocked(toast.warning).mockClear();
    vi.mocked(toast.info).mockClear();
    const { reportError } = await import("@/lib/reportError");
    (reportError as ReturnType<typeof vi.fn>).mockReset();
  });

  it("invoke sync_all_sources et invalide les queryKeys cibles apres succès", async () => {
    mockInvoke.mockResolvedValue([outcome("withings", 6)]);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    await runSync(qc);

    expect(mockInvoke).toHaveBeenCalledWith("sync_all_sources");
    const expectedKeys = [
      ["sources"],
      ["bodyMeasures"],
      ["bloodPressure"],
      ["sleep"],
      ["activity"],
      ["workouts"],
      ["heart"],
      ["devices"],
      ["insights"],
      ["sync_log"],
      ["dashboardCards"],
    ];
    for (const queryKey of expectedKeys) {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey });
    }
  });

  it("additionne les enregistrements de toutes les sources dans un seul message", async () => {
    mockInvoke.mockResolvedValue([outcome("withings", 6), outcome("oura", 4)]);

    await runSync(qc);

    expect(toast.success).toHaveBeenCalledWith(
      "Synchronisation terminée : 10 enregistrements importés"
    );
  });

  it("une source en échec est nommée et n'efface pas ce que les autres ont importé", async () => {
    mockInvoke.mockResolvedValue([
      outcome("oura", 0, ["Oura : refus du serveur"], true),
      outcome("withings", 6),
    ]);
    const { reportError } = await import("@/lib/reportError");

    await runSync(qc);

    expect(toast.success).not.toHaveBeenCalled();
    const message = vi.mocked(toast.warning).mock.calls[0]?.[0] as string;
    expect(message).toContain("6 enregistrements");
    expect(message).toContain("Oura : refus du serveur");
    expect(reportError).toHaveBeenCalledTimes(1);
    expect(reportError).toHaveBeenCalledWith(
      "Oura : refus du serveur",
      expect.objectContaining({ source: "sync", context: "sync_source:oura", toast: false })
    );
  });

  it("une erreur douce d'une source reste signalée sans rien taire du reste", async () => {
    mockInvoke.mockResolvedValue([outcome("withings", 3, ["Withings : Sommeil : 429"])]);
    const { reportError } = await import("@/lib/reportError");

    await runSync(qc);

    expect(toast.warning).toHaveBeenCalledTimes(1);
    expect(reportError).toHaveBeenCalledWith(
      "Withings : Sommeil : 429",
      expect.objectContaining({ context: "sync_source:withings", toast: false })
    );
  });

  it("dit qu'aucune source n'est connectée quand rien n'a été synchronisé", async () => {
    mockInvoke.mockResolvedValue([]);

    await runSync(qc);

    expect(toast.info).toHaveBeenCalledWith(
      "Aucune source connectée : connectez-en une dans Paramètres."
    );
    expect(toast.success).not.toHaveBeenCalled();
  });

  it("appelle reportError via catch sur exception franche (sans toast:false)", async () => {
    mockInvoke.mockRejectedValue(new Error("network failure"));
    const { reportError } = await import("@/lib/reportError");
    const mockReportError = reportError as ReturnType<typeof vi.fn>;

    await runSync(qc);

    expect(mockReportError).toHaveBeenCalledTimes(1);
    expect(mockReportError).toHaveBeenCalledWith(
      expect.any(Error),
      expect.objectContaining({ source: "sync", context: "sync_all_sources" })
    );
    const opts = (mockReportError.mock.calls[0] as [unknown, { toast?: boolean }])[1];
    expect(opts.toast).not.toBe(false);
  });
});

describe("useSync — progression", () => {
  it("affiche la source et l'étape reçues de sync-progress pendant la synchro", async () => {
    const qc = freshClient();
    useSyncStore.setState({ isSyncing: false, syncProgress: "" });
    let emitProgress: ((event: { payload: unknown }) => void) | undefined;
    vi.mocked(listen).mockImplementation(((
      event: string,
      handler: (event: { payload: unknown }) => void
    ) => {
      if (event === "sync-progress") emitProgress = handler;
      return Promise.resolve(vi.fn());
    }) as unknown as typeof listen);
    let progressDuringSync = "";
    mockInvoke.mockImplementation(async () => {
      emitProgress?.({ payload: { source: "withings", etape: 2, total: 7, libelle: "Activité" } });
      progressDuringSync = useSyncStore.getState().syncProgress;
      return [outcome("withings", 1)];
    });

    await runSync(qc);

    expect(progressDuringSync).toBe("Withings : Activité (2/7)");
    expect(useSyncStore.getState().syncProgress).toBe("");
  });

  it("invalide la lecture de la dernière synchro réussie à la fin", async () => {
    const qc = freshClient();
    const spy = vi.spyOn(qc, "invalidateQueries");
    useSyncStore.setState({ isSyncing: false, syncProgress: "" });
    mockInvoke.mockResolvedValue([outcome("withings", 1)]);

    await runSync(qc);

    expect(spy).toHaveBeenCalledWith({ queryKey: ["sync", "last-success"] });
  });

  it("avec un identifiant de source, ne synchronise que cette source", async () => {
    mockInvoke.mockResolvedValue([outcome("oura", 4)]);
    const { result } = renderHook(() => useSync("oura"), { wrapper: makeWrapper(freshClient()) });

    await act(async () => {
      await result.current.sync();
    });

    expect(mockInvoke).toHaveBeenCalledWith("sync_source", { source: "oura" });
    expect(mockInvoke).not.toHaveBeenCalledWith("sync_all_sources");
    expect(result.current.syncingHere).toBe(false);
  });

  it("marque la carte qui a lancé la synchro, pas les autres", async () => {
    let finish: (v: unknown) => void = () => undefined;
    mockInvoke.mockImplementation(() => new Promise((resolve) => (finish = resolve)));
    const mine = renderHook(() => useSync("oura"), { wrapper: makeWrapper(freshClient()) });
    const other = renderHook(() => useSync("polar"), { wrapper: makeWrapper(freshClient()) });

    let pending: Promise<void> = Promise.resolve();
    act(() => {
      pending = mine.result.current.sync();
    });

    expect(mine.result.current.syncingHere).toBe(true);
    expect(other.result.current.syncingHere).toBe(false);
    expect(other.result.current.isSyncing).toBe(true);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("sync_source", { source: "oura" }));
    await act(async () => {
      finish([]);
      await pending;
    });
    expect(mine.result.current.syncingHere).toBe(false);
  });
});
