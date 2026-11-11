import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { SourcesStep } from "./SourcesStep";
import type { SourceInfo } from "@/features/sources/types";

function source(over: Partial<SourceInfo> = {}): SourceInfo {
  return {
    id: "withings",
    label: "Withings",
    kind: "oauth",
    extensions: [],
    domains: ["body", "sleep"],
    connected: false,
    credentials_configured: false,
    last_sync_at: null,
    redirect_uri: null,
    developer_url: "https://developer.withings.com/",
    reauth: null,
    ...over,
  };
}

function renderStep(handlers: { onComplete?: () => void; onSkip?: () => void } = {}) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  return render(
    <QueryClientProvider client={client}>
      <SourcesStep
        onComplete={handlers.onComplete ?? vi.fn()}
        onSkip={handlers.onSkip ?? vi.fn()}
        titleId="titre-sources"
      />
    </QueryClientProvider>
  );
}

function answer(sources: () => Promise<unknown>) {
  vi.mocked(invoke).mockImplementation((cmd: string) =>
    cmd === "list_sources" ? sources() : Promise.resolve(null)
  );
}

describe("SourcesStep", () => {
  it("propose de choisir ses sources et de finir sans aucune", async () => {
    answer(() => Promise.resolve([source()]));
    const onSkip = vi.fn();
    const user = userEvent.setup();
    renderStep({ onSkip });

    expect(await screen.findByRole("heading", { name: "Choisir ses sources" })).toBeInTheDocument();
    expect(await screen.findByText("Withings")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Passer cette étape" }));
    expect(onSkip).toHaveBeenCalledTimes(1);
  });

  it("demande les identifiants et bloque la connexion tant qu'ils manquent", async () => {
    answer(() => Promise.resolve([source()]));
    const user = userEvent.setup();
    renderStep();

    await user.click(await screen.findByRole("button", { name: "Configurer Withings" }));

    expect(await screen.findByLabelText("Client ID")).toBeInTheDocument();
    expect(screen.getByLabelText("Client Secret")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Connecter mon compte" })).toBeDisabled();
  });

  it("propose la connexion quand les identifiants existent", async () => {
    answer(() => Promise.resolve([source({ credentials_configured: true })]));
    const user = userEvent.setup();
    renderStep();

    await user.click(await screen.findByRole("button", { name: "Configurer Withings" }));

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Connecter mon compte" })).toBeEnabled()
    );
    expect(screen.queryByLabelText("Client ID")).not.toBeInTheDocument();
  });

  it("signale une lecture impossible des sources et propose de réessayer", async () => {
    let calls = 0;
    answer(() => {
      calls += 1;
      return calls === 1
        ? Promise.reject("Erreur base de donnees : verrou")
        : Promise.resolve([source({ credentials_configured: true })]);
    });
    const user = userEvent.setup();
    renderStep();

    expect(await screen.findByText("Lecture des sources impossible.")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Réessayer" }));

    expect(await screen.findByRole("button", { name: "Configurer Withings" })).toBeInTheDocument();
  });

  it("montre une source déjà connectée et laisse continuer", async () => {
    answer(() => Promise.resolve([source({ connected: true, credentials_configured: true })]));
    const onComplete = vi.fn();
    const user = userEvent.setup();
    renderStep({ onComplete });

    expect(await screen.findByText("Connecté")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Continuer" }));
    expect(onComplete).toHaveBeenCalledTimes(1);
  });
});
