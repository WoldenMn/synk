import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";
import { invoke } from "@tauri-apps/api/core";
import { SourcesPanel } from "./index";

const WITHINGS = {
  id: "withings",
  label: "Withings",
  kind: "oauth",
  domains: ["body"],
  connected: true,
  credentials_configured: true,
  last_sync_at: null,
  redirect_uri: null,
  developer_url: null,
};
const OURA = { ...WITHINGS, id: "oura", label: "Oura", connected: false };

function renderPanel(sources: unknown[]) {
  vi.mocked(invoke).mockImplementation((cmd: string) => {
    if (cmd === "list_sources") return Promise.resolve(sources);
    if (cmd === "get_invalidated_secrets") return Promise.resolve({ sources: [] });
    if (cmd === "get_metric_priorities") return Promise.resolve([]);
    return Promise.resolve(null);
  });
  return render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfirmProvider>
        <SourcesPanel />
      </ConfirmProvider>
    </QueryClientProvider>
  );
}

describe("SourcesPanel", () => {
  it("montre une carte par source du registre", async () => {
    renderPanel([WITHINGS, OURA]);

    expect(await screen.findByRole("heading", { name: "Withings" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Oura" })).toBeInTheDocument();
  });

  it("range une source fichier dans le bloc d'import, jamais dans une carte de connexion", async () => {
    const apple = {
      ...WITHINGS,
      id: "apple_health",
      label: "Apple Santé",
      kind: "file",
      connected: false,
      credentials_configured: false,
      extensions: ["zip", "xml"],
    };
    renderPanel([WITHINGS, apple]);

    expect(await screen.findByText("Importer un fichier")).toBeInTheDocument();
    expect(screen.getByText("Apple Santé", { selector: "p" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Apple Santé" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Choisir un fichier/ })).toBeInTheDocument();
  });

  it("garde l'intervalle de synchronisation et l'onglet des sources actifs au départ", async () => {
    renderPanel([WITHINGS]);

    expect(await screen.findByRole("tab", { name: "Sources", selected: true })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Priorités", selected: false })).toBeInTheDocument();
    expect(screen.getByRole("tabpanel")).toBeInTheDocument();
    expect(screen.getByText("Synchronisation automatique")).toBeInTheDocument();
  });

  it("ouvre l'onglet des priorités", async () => {
    const user = userEvent.setup();
    renderPanel([WITHINGS]);

    await user.click(await screen.findByRole("tab", { name: "Priorités" }));

    expect(
      await screen.findByText(/Aucune mesure n'a encore plusieurs sources/)
    ).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Withings" })).not.toBeInTheDocument();
  });

  it("affiche l'appareil de Withings seulement quand Withings est connecté", async () => {
    renderPanel([{ ...WITHINGS, connected: false }]);

    await screen.findByRole("heading", { name: "Withings" });
    expect(screen.queryByText("Appareils Withings")).not.toBeInTheDocument();
  });

  it("va au dernier onglet avec Fin et au premier avec Début", async () => {
    renderPanel([WITHINGS]);
    const first = await screen.findByRole("tab", { name: "Sources" });
    first.focus();

    await userEvent.keyboard("{End}");
    expect(await screen.findByRole("tab", { name: "Priorités", selected: true })).toHaveFocus();

    await userEvent.keyboard("{Home}");
    expect(await screen.findByRole("tab", { name: "Sources", selected: true })).toHaveFocus();
  });
});
