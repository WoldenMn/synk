import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import { SourcesStatusBanner } from "./SourcesStatusBanner";
import type { SourceInfo } from "../types";

function source(over: Partial<SourceInfo>): SourceInfo {
  return {
    id: "withings",
    label: "Withings",
    kind: "oauth",
    extensions: [],
    domains: [],
    connected: false,
    credentials_configured: false,
    last_sync_at: null,
    redirect_uri: null,
    developer_url: null,
    reauth: null,
    ...over,
  };
}

function renderBanner(answer: () => Promise<unknown>) {
  vi.mocked(invoke).mockImplementation((cmd: string) =>
    cmd === "list_sources" ? answer() : Promise.resolve(null)
  );
  return render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <MemoryRouter>
        <SourcesStatusBanner />
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("SourcesStatusBanner", () => {
  it("prévient qu'aucune source n'est connectée et mène à Paramètres", async () => {
    renderBanner(() => Promise.resolve([source({})]));

    expect(await screen.findByText(/Aucune source n'est connectée/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Connecter une source" })).toBeInTheDocument();
  });

  it("se tait dès qu'une source est connectée", async () => {
    const { container } = renderBanner(() =>
      Promise.resolve([source({}), source({ id: "oura", label: "Oura", connected: true })])
    );

    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_sources"));
    await new Promise((r) => setTimeout(r, 30));
    expect(container).toBeEmptyDOMElement();
  });

  it("annonce la reconnexion d'une source connectée dont l'autorisation expire", async () => {
    renderBanner(() =>
      Promise.resolve([
        source({
          id: "google_health",
          label: "Google Health",
          connected: true,
          reauth: { status: "soon", expires_at: Date.UTC(2026, 10, 2, 12, 0, 0) / 1000 },
        }),
      ])
    );

    expect(
      await screen.findByText(/L'autorisation de Google Health expire le 2 nov\. 2026/)
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Ouvrir les sources" })).toBeInTheDocument();
    expect(screen.queryByText(/Aucune source n'est connectée/)).not.toBeInTheDocument();
  });

  it("dit que l'état est inconnu quand la lecture échoue", async () => {
    renderBanner(() => Promise.reject("Erreur base de donnees : verrou"));

    expect(await screen.findByText(/État des sources inconnu/)).toBeInTheDocument();
  });
});
