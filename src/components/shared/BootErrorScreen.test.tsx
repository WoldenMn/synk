import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import userEvent from "@testing-library/user-event";
import { BootErrorScreen } from "./BootErrorScreen";

const reloadSpy = vi.fn();
beforeEach(() => {
  vi.clearAllMocks();
  Object.defineProperty(window, "location", {
    writable: true,
    value: { ...window.location, reload: reloadSpy },
  });
});

describe("BootErrorScreen", () => {
  it("renders DbBusyAfterRetry with Réessayer + Ouvrir le dossier", () => {
    render(<BootErrorScreen error={{ kind: "DbBusyAfterRetry", attempts: 3 }} />);
    expect(screen.getByText(/verrouillée/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /réessayer/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /ouvrir le dossier/i })).toBeInTheDocument();
  });

  it("renders MigrationFailed with Restaurer la sauvegarde", () => {
    render(<BootErrorScreen error={{ kind: "MigrationFailed", version: 6, message: "x" }} />);
    expect(screen.getByText(/mise à jour de la base échouée \(v6\)/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /restaurer la sauvegarde/i })).toBeInTheDocument();
  });

  it("explique le refus d'un build de développement sur la vraie base", () => {
    render(<BootErrorScreen error={{ kind: "DevOnRealData", identifier: "com.synk.health" }} />);
    expect(
      screen.getByText("Mode développement refusé sur les vraies données")
    ).toBeInTheDocument();
    expect(screen.getByText(/npm run tauri:dev/)).toBeInTheDocument();
  });

  it("renders BackupFailed with restore and folder actions", () => {
    render(<BootErrorScreen error={{ kind: "BackupFailed", message: "disque plein" }} />);
    expect(screen.getByText(/sauvegarde avant mise à jour impossible/i)).toBeInTheDocument();
    expect(screen.getByText(/rien n'a été modifié/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /restaurer la sauvegarde/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /ouvrir le dossier/i })).toBeInTheDocument();
  });

  it("montre synk.db dans l'Explorateur", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      Promise.resolve(cmd === "get_app_data_dir" ? "C:\\Donnees\\com.synk.health" : null)
    );
    const user = userEvent.setup();
    render(<BootErrorScreen error={{ kind: "DbLocked", message: "verrou" }} />);
    await user.click(screen.getByRole("button", { name: /Ouvrir le dossier des données/ }));
    expect(revealItemInDir).toHaveBeenCalledWith("C:\\Donnees\\com.synk.health\\synk.db");
  });

  it("renders AppDataDir with Quitter button", () => {
    render(<BootErrorScreen error={{ kind: "AppDataDir", message: "Access denied" }} />);
    expect(screen.getByText(/inaccessible/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /quitter/i })).toBeInTheDocument();
  });

  it("relaunches the Rust boot before reloading when Réessayer is clicked", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    render(<BootErrorScreen error={{ kind: "DbLocked", message: "io error" }} />);
    fireEvent.click(screen.getByRole("button", { name: /réessayer/i }));
    await waitFor(() => expect(reloadSpy).toHaveBeenCalledOnce());
    expect(invoke).toHaveBeenCalledWith("retry_boot");
  });

  it("shows the reason and does not reload when retry_boot fails", async () => {
    vi.mocked(invoke).mockRejectedValue("Le démarrage n'est pas en échec");
    render(<BootErrorScreen error={{ kind: "DbLocked", message: "io error" }} />);
    fireEvent.click(screen.getByRole("button", { name: /réessayer/i }));
    expect(await screen.findByRole("status")).toHaveTextContent(/nouvelle tentative impossible/i);
    expect(reloadSpy).not.toHaveBeenCalled();
  });

  it("writes the text of a typed Rust error, never [object Object]", async () => {
    vi.mocked(invoke).mockRejectedValue({
      kind: "Validation",
      message: "Erreur de validation : le démarrage n'est pas en échec",
      detail: "le démarrage n'est pas en échec",
    });
    render(<BootErrorScreen error={{ kind: "DbLocked", message: "io error" }} />);
    fireEvent.click(screen.getByRole("button", { name: /réessayer/i }));
    const status = await screen.findByRole("status");
    expect(status).toHaveTextContent(/nouvelle tentative impossible/i);
    expect(status).toHaveTextContent("Erreur de validation : le démarrage n'est pas en échec");
    expect(status).not.toHaveTextContent("[object Object]");
  });

  it("restores, then relaunches the boot", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) =>
      cmd === "restore_latest_backup"
        ? {
            restored_from: "b.db",
            broken_kept_at: "synk.db.broken-x",
            skipped: ["a.db (corrompue)"],
          }
        : undefined
    );
    render(<BootErrorScreen error={{ kind: "MigrationFailed", version: 6, message: "x" }} />);
    fireEvent.click(screen.getByRole("button", { name: /restaurer la sauvegarde/i }));
    await waitFor(() => expect(reloadSpy).toHaveBeenCalledOnce());
    expect(vi.mocked(invoke).mock.calls.map((c) => c[0])).toEqual([
      "restore_latest_backup",
      "retry_boot",
    ]);
    expect(screen.getByRole("status")).toHaveTextContent(/1 sauvegarde\(s\) illisible/);
  });

  it("expands technical détails on click", () => {
    render(
      <BootErrorScreen error={{ kind: "MigrationFailed", version: 6, message: "table missing" }} />
    );
    fireEvent.click(screen.getByText(/voir les détails techniques/i));
    expect(screen.getByText(/"version": 6/)).toBeInTheDocument();
  });

  it("renders CryptoUnavailable variant with helpful message", () => {
    render(
      <BootErrorScreen error={{ kind: "CryptoUnavailable", message: "keyring init failed" }} />
    );
    expect(screen.getByText(/stockage sécurisé inaccessible/i)).toBeInTheDocument();
    expect(screen.getByText(/Credential Manager/i)).toBeInTheDocument();
  });

  it("renders CryptoMigrationFailed variant with backup hint", () => {
    render(<BootErrorScreen error={{ kind: "CryptoMigrationFailed", message: "db error" }} />);
    expect(screen.getByText(/chiffrement des accès aux sources/i)).toBeInTheDocument();
    expect(screen.getByText(/sauvegarde automatique/i)).toBeInTheDocument();
  });
});
