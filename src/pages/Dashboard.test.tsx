import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render, screen } from "@testing-library/react";
import { QueryClientProvider, type QueryClient } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { createQueryClient } from "@/lib/queryClient";
import Dashboard from "./Dashboard";

const mockInvoke = vi.mocked(invoke);

const NEUTRAL_TREND = {
  current_value: null,
  previous_value: null,
  percent_change: null,
  direction: "stable",
};

const EMPTY_CARDS = {
  body: null,
  blood_pressure: null,
  sleep: null,
  activity: null,
  workout: null,
  heart: null,
  trends: {
    weight: NEUTRAL_TREND,
    systolic: NEUTRAL_TREND,
    sleep_score: NEUTRAL_TREND,
    steps: NEUTRAL_TREND,
  },
};

function answer(cmd: string): unknown {
  if (cmd.startsWith("get_latest")) return null;
  if (cmd === "get_dashboard_cards") return EMPTY_CARDS;
  return [];
}

function renderDashboard(client: QueryClient) {
  return render(
    <QueryClientProvider client={client}>
      <MemoryRouter>
        <Dashboard />
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("Dashboard", () => {
  afterEach(() => vi.useRealTimers());

  it("ne relance aucune lecture au retour dans les cinq minutes", async () => {
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date(2026, 9, 3, 9, 0, 0));
    mockInvoke.mockReset();
    mockInvoke.mockImplementation((cmd: string) => Promise.resolve(answer(cmd)));
    const client = createQueryClient();

    renderDashboard(client);
    await vi.waitFor(() =>
      expect(mockInvoke.mock.calls.some((c) => c[0] === "get_goals")).toBe(true)
    );
    await act(async () => {
      await new Promise((r) => setTimeout(r, 50));
    });
    const firstVisit = mockInvoke.mock.calls.length;
    cleanup();

    vi.setSystemTime(new Date(2026, 9, 3, 9, 1, 0));
    renderDashboard(client);
    await act(async () => {
      await new Promise((r) => setTimeout(r, 50));
    });

    expect(mockInvoke.mock.calls.length).toBe(firstVisit);
  });

  it("lit le tableau de bord en onze lectures au plus au premier affichage", async () => {
    mockInvoke.mockReset();
    mockInvoke.mockImplementation((cmd: string) => Promise.resolve(answer(cmd)));

    renderDashboard(createQueryClient());
    await vi.waitFor(() =>
      expect(mockInvoke.mock.calls.some((c) => c[0] === "get_goals")).toBe(true)
    );
    await act(async () => {
      await new Promise((r) => setTimeout(r, 50));
    });

    const commands = mockInvoke.mock.calls.map((c) => c[0] as string);
    expect(commands.length).toBeLessThanOrEqual(11);
    expect(commands.filter((c) => c === "get_dashboard_cards")).toHaveLength(1);
    expect(commands.some((c) => c.startsWith("get_latest") || c === "get_metric_trend")).toBe(
      false
    );
  });

  it("fait de chaque carte un lien vers la page de sa mesure", async () => {
    mockInvoke.mockReset();
    mockInvoke.mockImplementation((cmd: string) => Promise.resolve(answer(cmd)));
    renderDashboard(createQueryClient());
    const pages: Record<string, string> = {
      Poids: "/body",
      Tension: "/blood-pressure",
      Sommeil: "/sleep",
      Activité: "/activity",
      Entraînement: "/workouts",
      "Cœur / ECG": "/heart",
    };
    for (const [title, href] of Object.entries(pages)) {
      const link = await screen.findByRole("link", { name: new RegExp(`^${title}`) });
      expect(link).toHaveAttribute("href", href);
    }
  });
});
