import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import Insights from "./Insights";

const insight = (id: number, severity: "info" | "warning" | "danger") => ({
  id,
  kind: "weight_trend",
  severity,
  metric: "weight",
  title: `Insight ${id}`,
  message: "Message",
  value: null,
  delta: null,
  generated_at: "2026-10-01T09:30:00Z",
  seen: 1,
});

describe("Insights", () => {
  it("dit la gravité en mots, pas seulement par l'icône et la teinte", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      Promise.resolve(
        cmd === "get_insights"
          ? [insight(1, "info"), insight(2, "warning"), insight(3, "danger")]
          : null
      )
    );
    render(
      <QueryClientProvider client={new QueryClient()}>
        <MemoryRouter>
          <Insights />
        </MemoryRouter>
      </QueryClientProvider>
    );
    expect(await screen.findByText("Information")).toBeInTheDocument();
    expect(screen.getByText("Avertissement")).toBeInTheDocument();
    expect(screen.getByText("Critique")).toBeInTheDocument();
  });

  it("nomme l'état vide en français courant, sans le mot insight", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      Promise.resolve(cmd === "get_insights" ? [] : null)
    );
    render(
      <QueryClientProvider client={new QueryClient()}>
        <MemoryRouter>
          <Insights />
        </MemoryRouter>
      </QueryClientProvider>
    );
    expect(await screen.findByText("Aucun constat pour l'instant")).toBeInTheDocument();
  });
});
