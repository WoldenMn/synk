import { describe, expect, it, vi } from "vitest";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

vi.mock("@/features/sources/api", () => ({
  sourceStartAuth: vi.fn(),
  sourceDisconnect: vi.fn(() =>
    Promise.reject(new Error("Erreur base de donnees : disk I/O error"))
  ),
  sourceSetCredentials: vi.fn(),
  listSources: vi.fn(() => Promise.resolve([])),
}));
const useSyncMock = vi.hoisted(() =>
  vi.fn((_source?: string) => ({ sync: vi.fn(), isSyncing: false, syncingHere: false }))
);
vi.mock("@/hooks/useSync", () => ({ useSync: useSyncMock }));
vi.mock("@/lib/reportError", () => ({ reportError: vi.fn() }));
vi.mock("sonner", () => ({ toast: { warning: vi.fn(), success: vi.fn(), error: vi.fn() } }));

import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";
import { sourceDisconnect, sourceStartAuth } from "@/features/sources/api";
import type { SourceInfo } from "@/features/sources/types";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";
import { SourceCard } from "./SourceCard";

const WITHINGS: SourceInfo = {
  id: "withings",
  label: "Withings",
  kind: "oauth",
  extensions: [],
  domains: ["body", "sleep"],
  connected: false,
  credentials_configured: true,
  last_sync_at: null,
  redirect_uri: "http://localhost:17483/callback",
  developer_url: "https://developer.withings.com/",
  reauth: null,
};

function renderCard(source: SourceInfo) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <ConfirmProvider>
        <SourceCard source={source} />
      </ConfirmProvider>
    </QueryClientProvider>
  );
}

async function disconnect() {
  await userEvent.click(screen.getByRole("button", { name: "Déconnecter Withings" }));
  const dialog = await screen.findByRole("dialog");
  await userEvent.click(within(dialog).getByRole("button", { name: "Déconnecter" }));
}

describe("SourceCard", () => {
  it("demande confirmation avant de déconnecter, dit ce qui est effacé et ce qui reste", async () => {
    vi.mocked(sourceDisconnect).mockClear();
    renderCard({ ...WITHINGS, connected: true });

    await userEvent.click(screen.getByRole("button", { name: "Déconnecter Withings" }));

    const dialog = await screen.findByRole("dialog");
    expect(dialog).toHaveTextContent("Déconnecter Withings ?");
    expect(dialog).toHaveTextContent(/synchronisation initiale/);
    expect(dialog).toHaveTextContent(/mesures déjà importées restent/);
    expect(dialog).toHaveTextContent(/révoquer/);
    expect(sourceDisconnect).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole("button", { name: "Annuler" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(sourceDisconnect).not.toHaveBeenCalled();
  });

  it("affiche l'échéance d'une autorisation qui expire, seulement pour une source connectée", () => {
    const reauth = {
      status: "expired" as const,
      expires_at: Date.UTC(2026, 10, 2, 12, 0, 0) / 1000,
    };
    const { unmount } = renderCard({ ...WITHINGS, connected: true, reauth });
    expect(screen.getByRole("status")).toHaveTextContent(
      "L'autorisation de Withings a expiré le 2 nov. 2026"
    );
    unmount();

    renderCard({ ...WITHINGS, connected: false, reauth });
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("décrit la source : domaines couverts, dernière synchronisation, état", () => {
    renderCard({ ...WITHINGS, connected: true, last_sync_at: "2026-10-25T08:00:05Z" });

    expect(screen.getByRole("heading", { name: "Withings" })).toBeInTheDocument();
    expect(screen.getByText("Connecté")).toBeInTheDocument();
    expect(screen.getByText(/Composition corporelle/)).toBeInTheDocument();
    expect(screen.getByText(/Sommeil/)).toBeInTheDocument();
    expect(screen.getByText(/Dernière synchro : 25 oct\. 2026/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Synchroniser Withings" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Déconnecter Withings" })).toBeEnabled();
  });

  it("ne synchronise que sa propre source et n'anime que son bouton", () => {
    useSyncMock.mockReturnValue({ sync: vi.fn(), isSyncing: true, syncingHere: false });
    renderCard({ ...WITHINGS, connected: true });

    expect(useSyncMock).toHaveBeenCalledWith("withings");
    expect(screen.getByRole("button", { name: "Synchroniser Withings" })).toBeDisabled();

    useSyncMock.mockReturnValue({ sync: vi.fn(), isSyncing: true, syncingHere: true });
    renderCard({ ...WITHINGS, id: "oura", label: "Oura", connected: true });
    expect(screen.getByRole("button", { name: "Synchronisation... Oura" })).toBeDisabled();
    useSyncMock.mockReturnValue({ sync: vi.fn(), isSyncing: false, syncingHere: false });
  });

  it("donne à chaque bouton répété le nom de sa source", () => {
    renderCard({ ...WITHINGS, connected: true });
    renderCard({ ...WITHINGS, id: "oura", label: "Oura", connected: true });

    for (const action of ["Synchroniser", "Déconnecter"]) {
      expect(screen.getByRole("button", { name: `${action} Withings` })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: `${action} Oura` })).toBeInTheDocument();
    }
  });

  it("dit « jamais » d'une source qui n'a pas encore synchronisé", () => {
    renderCard({ ...WITHINGS, connected: true });
    expect(screen.getByText(/Jamais synchronisée/)).toBeInTheDocument();
  });

  it("bloque la connexion tant que les identifiants manquent", () => {
    renderCard({ ...WITHINGS, credentials_configured: false });

    expect(
      screen.getByRole("button", { name: "Saisissez d'abord les identifiants Withings" })
    ).toBeDisabled();
    expect(screen.getByLabelText("Client ID")).toBeInTheDocument();
  });

  it("donne l'URL de rappel à déclarer et le lien du portail développeur", () => {
    renderCard(WITHINGS);

    expect(screen.getByText("http://localhost:17483/callback")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /developer\.withings\.com/ })).toHaveAttribute(
      "href",
      "https://developer.withings.com/"
    );
  });

  it("signale l'échec de la déconnexion de la source nommée", async () => {
    renderCard({ ...WITHINGS, connected: true });

    await disconnect();

    await waitFor(() => expect(reportError).toHaveBeenCalled());
    expect(reportError).toHaveBeenCalledWith(expect.any(Error), {
      source: "manual",
      context: "source_disconnect:withings",
    });
  });

  it("nomme l'échec de la révocation chez la marque, la déconnexion locale étant faite", async () => {
    vi.mocked(sourceDisconnect).mockResolvedValueOnce({
      revoked: false,
      warning: "Polar : révocation impossible, la marque n'a pas répondu",
    });
    vi.mocked(toast.warning).mockClear();
    renderCard({ ...WITHINGS, connected: true });

    await disconnect();

    await waitFor(() =>
      expect(toast.warning).toHaveBeenCalledWith(
        expect.stringContaining("Polar : révocation impossible"),
        expect.anything()
      )
    );
    expect(reportError).not.toHaveBeenCalled();
  });

  it("ne dit rien de plus quand la révocation a réussi", async () => {
    vi.mocked(sourceDisconnect).mockResolvedValueOnce({ revoked: true, warning: null });
    vi.mocked(toast.warning).mockClear();
    vi.mocked(reportError).mockClear();
    renderCard({ ...WITHINGS, connected: true });

    await disconnect();

    await waitFor(() => expect(sourceDisconnect).toHaveBeenCalled());
    expect(toast.warning).not.toHaveBeenCalled();
  });

  describe("évènements de connexion", () => {
    const handlers = new Map<string, (e: { payload: unknown }) => void>();

    async function startConnecting() {
      handlers.clear();
      vi.mocked(listen).mockImplementation(async (name, cb) => {
        handlers.set(name, cb as (e: { payload: unknown }) => void);
        return () => undefined;
      });
      vi.mocked(sourceStartAuth).mockResolvedValue("https://account.withings.com/oauth2");
      vi.mocked(reportError).mockClear();
      vi.mocked(reportError).mockReturnValue({ userMessage: "erreur" } as ReturnType<
        typeof reportError
      >);
      vi.mocked(toast.warning).mockClear();
      renderCard(WITHINGS);
      await waitFor(() => expect(handlers.has("withings-auth-timeout")).toBe(true));
      await userEvent.click(screen.getByRole("button", { name: "Connecter Withings" }));
      expect(await screen.findByRole("button", { name: "En attente... Withings" })).toBeDisabled();
      expect(sourceStartAuth).toHaveBeenCalledWith("withings");
    }

    it("rend le bouton et prévient après le délai de connexion", async () => {
      await startConnecting();

      act(() => handlers.get("withings-auth-timeout")?.({ payload: null }));

      expect(await screen.findByRole("button", { name: "Connecter Withings" })).toBeEnabled();
      expect(toast.warning).toHaveBeenCalledWith("Délai d'autorisation Withings dépassé.");
    });

    it("rend le bouton et signale l'erreur typée de connexion", async () => {
      await startConnecting();
      const payload = {
        kind: "Unauthorized",
        message: "Acces expire : refuse",
        detail: "refuse",
      };

      act(() => handlers.get("withings-auth-error")?.({ payload }));

      expect(await screen.findByRole("button", { name: "Connecter Withings" })).toBeEnabled();
      expect(reportError).toHaveBeenCalledWith(payload, {
        source: "manual",
        context: "withings_auth",
      });
    });

    it("montre l'erreur de connexion dans la carte et laisse réessayer", async () => {
      await startConnecting();
      vi.mocked(reportError).mockReturnValue({
        userMessage: "Accès refusé par Withings",
      } as ReturnType<typeof reportError>);

      act(() => handlers.get("withings-auth-error")?.({ payload: { kind: "Forbidden" } }));

      expect(await screen.findByRole("alert")).toHaveTextContent("Accès refusé par Withings");
      expect(screen.getByRole("button", { name: "Connecter Withings" })).toBeEnabled();
    });

    it("permet d'annuler l'attente de l'autorisation", async () => {
      await startConnecting();

      await userEvent.click(screen.getByRole("button", { name: "Annuler la connexion Withings" }));

      expect(await screen.findByRole("button", { name: "Connecter Withings" })).toBeEnabled();
      expect(
        screen.queryByRole("button", { name: "Annuler la connexion Withings" })
      ).not.toBeInTheDocument();
    });

    it("annonce la réussite de la connexion et rend le bouton", async () => {
      await startConnecting();
      vi.mocked(toast.success).mockClear();

      act(() => handlers.get("withings-auth-success")?.({ payload: null }));

      expect(toast.success).toHaveBeenCalledWith("Connexion Withings réussie !");
      expect(await screen.findByRole("button", { name: "Connecter Withings" })).toBeEnabled();
    });

    it("s'abonne une seule fois à chaque évènement de connexion, puis se désabonne au démontage", async () => {
      const off = vi.fn();
      vi.mocked(listen).mockClear();
      vi.mocked(listen).mockImplementation(async () => off);
      const { unmount } = renderCard(WITHINGS);
      await waitFor(() => expect(listen).toHaveBeenCalledTimes(3));
      expect(
        vi
          .mocked(listen)
          .mock.calls.map(([name]) => name)
          .sort()
      ).toEqual(["withings-auth-error", "withings-auth-success", "withings-auth-timeout"]);
      unmount();
      await waitFor(() => expect(off).toHaveBeenCalledTimes(3));
    });
  });
});
