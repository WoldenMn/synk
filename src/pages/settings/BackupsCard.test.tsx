import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { BackupsCard } from "./BackupsCard";

function renderCard() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <BackupsCard />
    </QueryClientProvider>
  );
}

const DAILY = {
  path: "C:\\d\\backups\\daily\\synk_2026-10-03_08-00-00_quotidienne.db",
  kind: "quotidienne",
  size_bytes: 1153434,
  modified: "2026-10-03T06:00:00Z",
};

describe("BackupsCard", () => {
  it("liste les sauvegardes et en crée une à la demande", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "list_backups") return Promise.resolve([DAILY]);
      if (cmd === "create_backup")
        return Promise.resolve("C:\\d\\backups\\synk_2026-10-03_09-00-00.db");
      return Promise.resolve(null);
    });
    const user = userEvent.setup();
    renderCard();
    expect(await screen.findByText(/Quotidienne/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Créer une sauvegarde maintenant" }));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("create_backup"));
  });

  it("affiche l'erreur de lecture avec Réessayer", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      cmd === "list_backups"
        ? Promise.reject("Erreur interne : dossier illisible")
        : Promise.resolve(null)
    );
    renderCard();
    expect(await screen.findByText(/dossier illisible/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Réessayer" })).toBeInTheDocument();
  });
});
