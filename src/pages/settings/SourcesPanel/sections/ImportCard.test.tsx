import { describe, expect, it, vi } from "vitest";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

vi.mock("@/features/sources/api", () => ({
  importSourceFile: vi.fn(),
  cancelImport: vi.fn(() => Promise.resolve(true)),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@/lib/reportError", () => ({ reportError: vi.fn() }));
vi.mock("sonner", () => ({
  toast: { warning: vi.fn(), success: vi.fn(), error: vi.fn(), info: vi.fn() },
}));

import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";
import { cancelImport, importSourceFile } from "@/features/sources/api";
import type { ImportReport, SourceInfo } from "@/features/sources/types";
import { ImportCard } from "./ImportCard";

const APPLE: SourceInfo = {
  id: "apple_health",
  label: "Apple Santé",
  kind: "file",
  domains: ["body", "sleep"],
  connected: false,
  credentials_configured: false,
  last_sync_at: null,
  redirect_uri: null,
  developer_url: null,
  reauth: null,
  extensions: ["zip", "xml"],
};

const REPORT: ImportReport = {
  source: "apple_health",
  cancelled: false,
  tally: {
    observations: 12,
    observations_ignored: 0,
    blood_pressure: 0,
    sleep: 2,
    activity: 0,
    workouts: 0,
    devices: {},
  },
  skipped: [{ reason: "unité inconnue : furlong", count: 2 }],
  duration_ms: 10,
};

function renderCard(sources: SourceInfo[] = [APPLE]) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const spy = vi.spyOn(client, "invalidateQueries");
  render(
    <QueryClientProvider client={client}>
      <ImportCard sources={sources} />
    </QueryClientProvider>
  );
  return spy;
}

describe("ImportCard", () => {
  it("ne s'affiche pas sans source fichier", () => {
    renderCard([]);
    expect(screen.queryByText("Importer un fichier")).not.toBeInTheDocument();
  });

  it("donne à chaque bouton « Choisir un fichier » le nom de sa source", () => {
    renderCard([APPLE, { ...APPLE, id: "fit", label: "Fichiers FIT", extensions: ["fit"] }]);

    expect(
      screen.getByRole("button", { name: "Choisir un fichier Apple Santé" })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Choisir un fichier Fichiers FIT" })
    ).toBeInTheDocument();
  });

  it("filtre le sélecteur sur les extensions de la source et ne lance rien si on referme", async () => {
    vi.mocked(open).mockResolvedValue(null);
    renderCard();

    await userEvent.click(screen.getByRole("button", { name: /Choisir un fichier/ }));

    expect(open).toHaveBeenCalledWith(
      expect.objectContaining({
        filters: [{ name: "Apple Santé", extensions: ["zip", "xml"] }],
        multiple: false,
      })
    );
    expect(importSourceFile).not.toHaveBeenCalled();
  });

  it("importe le fichier choisi, résume le résultat et rafraîchit les données", async () => {
    vi.mocked(open).mockResolvedValue("C:/export.zip");
    vi.mocked(importSourceFile).mockResolvedValue(REPORT);
    const invalidate = renderCard();

    await userEvent.click(screen.getByRole("button", { name: /Choisir un fichier/ }));

    await waitFor(() =>
      expect(importSourceFile).toHaveBeenCalledWith("apple_health", "C:/export.zip")
    );
    expect(await screen.findByText("12 mesures, 2 nuits")).toBeInTheDocument();
    expect(screen.getByText(/unité inconnue : furlong \(2\)/)).toBeInTheDocument();
    expect(toast.success).toHaveBeenCalledWith("Import terminé : 12 mesures, 2 nuits");
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ["sleep"] });
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ["metric-series"] });
  });

  it("montre la progression reçue et permet d'annuler pendant l'import", async () => {
    vi.mocked(open).mockResolvedValue("C:/export.zip");
    let finish: (r: ImportReport) => void = () => undefined;
    vi.mocked(importSourceFile).mockReturnValue(new Promise((res) => (finish = res)));
    let push: (payload: unknown) => void = () => undefined;
    vi.mocked(listen).mockImplementation(async (_name, cb) => {
      push = (payload) => cb({ event: "import-progress", id: 1, payload } as never);
      return () => undefined;
    });
    renderCard();

    await userEvent.click(screen.getByRole("button", { name: /Choisir un fichier/ }));
    await screen.findByRole("progressbar");
    act(() => push({ source: "apple_health", percent: 42, records: 1500 }));

    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "42");
    expect(screen.getByText(/42 %, 1500 enregistrements/)).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Annuler l'import Apple Santé" }));
    expect(cancelImport).toHaveBeenCalled();

    await act(async () => finish({ ...REPORT, cancelled: true }));
    expect(toast.info).toHaveBeenCalledWith("Import annulé : rien n'a été écrit.");
    expect(screen.queryByRole("progressbar")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Choisir un fichier/ })).toBeInTheDocument();
  });

  it("n'annonce l'avancement qu'aux paliers de 10 % pour ne pas saturer un lecteur d'écran", async () => {
    vi.mocked(open).mockResolvedValue("C:/export.zip");
    vi.mocked(importSourceFile).mockReturnValue(new Promise(() => undefined));
    let push: (payload: unknown) => void = () => undefined;
    vi.mocked(listen).mockImplementation(async (_name, cb) => {
      push = (payload) => cb({ event: "import-progress", id: 1, payload } as never);
      return () => undefined;
    });
    renderCard();
    await userEvent.click(screen.getByRole("button", { name: /Choisir un fichier/ }));
    await screen.findByRole("progressbar");
    const announced = () =>
      screen
        .getAllByRole("status")
        .map((node) => node.textContent ?? "")
        .join("|");

    act(() => push({ source: "apple_health", percent: 3, records: 10 }));
    expect(announced()).not.toMatch(/Import Apple Santé : [0-9]/);
    act(() => push({ source: "apple_health", percent: 12, records: 100 }));
    expect(announced()).toMatch(/Import Apple Santé : 10 %/);
    act(() => push({ source: "apple_health", percent: 18, records: 150 }));
    expect(announced()).toMatch(/Import Apple Santé : 10 %/);
    act(() => push({ source: "apple_health", percent: 25, records: 200 }));
    expect(announced()).toMatch(/Import Apple Santé : 20 %/);
  });

  it("monte la zone d'annonce avant le résultat pour qu'il soit lu", async () => {
    vi.mocked(open).mockResolvedValue("C:/export.zip");
    vi.mocked(importSourceFile).mockResolvedValue(REPORT);
    renderCard();

    const regions = screen.getAllByRole("status").filter((node) => node.textContent === "");
    expect(regions.length).toBeGreaterThan(0);

    await userEvent.click(screen.getByRole("button", { name: /Choisir un fichier/ }));

    await waitFor(() =>
      expect(regions.some((node) => node.textContent === "12 mesures, 2 nuits")).toBe(true)
    );
  });

  it("signale l'échec et redonne la main", async () => {
    vi.mocked(open).mockResolvedValue("C:/export.zip");
    vi.mocked(importSourceFile).mockRejectedValue(new Error("illisible"));
    renderCard();

    await userEvent.click(screen.getByRole("button", { name: /Choisir un fichier/ }));

    await waitFor(() => expect(reportError).toHaveBeenCalled());
    expect(screen.getByRole("button", { name: /Choisir un fichier/ })).toBeEnabled();
  });
});
