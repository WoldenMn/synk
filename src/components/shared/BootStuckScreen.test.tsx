import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { BootStuckScreen } from "./BootStuckScreen";

vi.mock("@/hooks/appReady/api", () => ({
  getAppDataDir: vi.fn(() => Promise.resolve("C:/data")),
  getBootState: vi.fn(() => Promise.resolve({ status: "loading" })),
}));
vi.mock("@/lib/reveal", () => ({ revealDataFolder: vi.fn(() => Promise.resolve()) }));

describe("BootStuckScreen", () => {
  it("parle français courant, sans jargon ni mot sans accent", () => {
    render(<BootStuckScreen lastSnapshot={{ status: "loading" }} lastError={null} />);
    expect(
      screen.getByRole("heading", { name: "Synk ne termine pas son démarrage" })
    ).toBeInTheDocument();
    const text = document.body.textContent ?? "";
    expect(text).not.toMatch(/\bpret\b|cote Rust|apres rechargement|\bPRET\b|ECHOUE/);
    expect(text).not.toMatch(/boot Rust|moteur Rust/);
  });

  it("range le détail technique dans un volet replié pour le support", () => {
    render(
      <BootStuckScreen
        lastSnapshot={{
          status: "failed",
          error: { kind: "DbLocked", message: "base verrouillee" },
        }}
        lastError="get_boot_state a échoué"
      />
    );
    const summary = screen.getByText("Détails pour le support");
    const details = summary.closest("details");
    expect(details).not.toBeNull();
    expect(details?.open).toBe(false);
    expect(details).toHaveTextContent("DbLocked");
    const shown = screen.getAllByText(/DbLocked/);
    expect(shown.length).toBeGreaterThan(0);
    for (const el of shown) expect(el.closest("details")).toBe(details);
  });

  it("dit en mots ce qui se passe quand le démarrage est terminé côté moteur", () => {
    render(
      <BootStuckScreen
        lastSnapshot={{ status: "ready", integrity_warning: null }}
        lastError={null}
      />
    );
    expect(screen.getByText(/est prêt, mais l'écran/)).toBeInTheDocument();
  });
});
