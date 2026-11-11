import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";
import Export from "./Export";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), info: vi.fn(), warning: vi.fn() },
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <ConfirmProvider>
        <Export />
      </ConfirmProvider>
    </QueryClientProvider>
  );
}

describe("Export", () => {
  it("prévient que les fichiers exportés ne sont pas chiffrés", () => {
    renderPage();
    expect(screen.getByText(/ne sont pas chiffrés/)).toBeInTheDocument();
  });

  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(toast.success).mockReset();
    vi.mocked(toast.info).mockReset();
    vi.mocked(toast.warning).mockReset();
    vi.mocked(open).mockReset();
  });

  it("dit qu'il n'y a rien à exporter au lieu d'annoncer un succès", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      Promise.resolve(cmd === "export_pdf" ? { path: null, rows: 0 } : null)
    );
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByRole("button", { name: /Exporter PDF/ }));
    expect(toast.info).toHaveBeenCalledWith(
      "Aucune donnée sur cette période : rien n'a été exporté."
    );
    expect(toast.success).not.toHaveBeenCalled();
  });

  it("exporte tout l'historique par défaut", async () => {
    vi.mocked(invoke).mockResolvedValue({ path: "C:\\x.csv", rows: 3 });
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByRole("button", { name: /Exporter CSV/ }));
    expect(invoke).toHaveBeenCalledWith(
      "export_csv",
      expect.objectContaining({ from: "1970-01-01T00:00:00.000Z" })
    );
  });

  it("annonce le nombre de lignes exportées", async () => {
    vi.mocked(invoke).mockResolvedValue({ path: "C:\\x.csv", rows: 3 });
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByRole("button", { name: /Exporter CSV/ }));
    expect(toast.success).toHaveBeenCalledWith(
      expect.stringContaining("3 lignes"),
      expect.objectContaining({ description: "C:\\x.csv" })
    );
  });

  it("liste le contenu du fichier avant d'importer, puis annonce le bilan", async () => {
    vi.mocked(open).mockResolvedValue("C:\\sauvegarde.json");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "inspect_import_file") {
        return Promise.resolve([
          { key: "body_measures", rows: 12 },
          { key: "medications", rows: 2 },
        ]);
      }
      if (cmd === "import_json") {
        return Promise.resolve({
          tables: [
            { key: "body_measures", inserted: 0, ignored: 12, updated: 0, orphans: 0 },
            { key: "user_profile", inserted: 0, ignored: 0, updated: 1, orphans: 0 },
          ],
        });
      }
      return Promise.resolve(null);
    });
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByRole("button", { name: /Importer JSON/ }));
    expect(await screen.findByText("Importer ce fichier ?")).toBeInTheDocument();
    expect(screen.getByText(/12 pesées/)).toBeInTheDocument();
    expect(screen.getByText(/2 médicaments/)).toBeInTheDocument();
    expect(invoke).not.toHaveBeenCalledWith("import_json", expect.anything());
    await user.click(screen.getByRole("button", { name: "Importer" }));
    expect(invoke).toHaveBeenCalledWith("import_json", { path: "C:\\sauvegarde.json" });
    expect(toast.success).toHaveBeenCalledWith(
      "Import terminé : 0 ajoutée, 12 déjà présentes, 1 complétée"
    );
  });

  it("n'importe rien si l'utilisateur annule", async () => {
    vi.mocked(open).mockResolvedValue("C:\\sauvegarde.json");
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      Promise.resolve(cmd === "inspect_import_file" ? [{ key: "goals", rows: 1 }] : null)
    );
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByRole("button", { name: /Importer JSON/ }));
    await user.click(await screen.findByRole("button", { name: "Annuler" }));
    expect(invoke).not.toHaveBeenCalledWith("import_json", expect.anything());
  });
});
