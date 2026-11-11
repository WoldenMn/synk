import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import { ConfirmProvider } from "@/components/ui/confirm-dialog";
import Timeline from "./Timeline";

const entry = (id: number, status: string) => ({
  id,
  source: id === 2 ? "oura" : "withings",
  started_at: "2026-10-01T09:30:00Z",
  finished_at: null,
  data_type: `type${id}`,
  duration_ms: 1200,
  records_added: 0,
  status,
  error: null,
});

describe("Timeline", () => {
  it("dit le statut de chaque synchronisation en mots", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      Promise.resolve(
        cmd === "get_sync_log"
          ? [entry(1, "success"), entry(2, "error"), entry(3, "running")]
          : cmd === "get_error_log"
            ? []
            : null
      )
    );
    render(
      <QueryClientProvider client={new QueryClient()}>
        <ConfirmProvider>
          <MemoryRouter>
            <Timeline />
          </MemoryRouter>
        </ConfirmProvider>
      </QueryClientProvider>
    );
    expect(await screen.findByText("Réussie")).toBeInTheDocument();
    expect(screen.getByText("Échec")).toBeInTheDocument();
    expect(screen.getByText("En cours")).toBeInTheDocument();
  });

  it("nomme la source de chaque synchronisation", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      Promise.resolve(
        cmd === "get_sync_log"
          ? [entry(1, "success"), entry(2, "error")]
          : cmd === "get_error_log"
            ? []
            : null
      )
    );
    render(
      <QueryClientProvider client={new QueryClient()}>
        <ConfirmProvider>
          <MemoryRouter>
            <Timeline />
          </MemoryRouter>
        </ConfirmProvider>
      </QueryClientProvider>
    );
    expect(await screen.findByText("Withings")).toBeInTheDocument();
    expect(screen.getByText("Oura")).toBeInTheDocument();
  });
});
