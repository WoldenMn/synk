import { describe, expect, it, vi } from "vitest";
import { cloneElement, isValidElement } from "react";
import type { ReactElement } from "react";
import { render, screen, waitFor } from "@testing-library/react";
import { ActivityChart } from "./ActivityChart";
import { SleepChart } from "./SleepChart";
import type { Activity } from "@/features/activity/types";
import type { SleepRecord } from "@/features/sleep/types";

vi.mock("recharts", async () => {
  const actual = await vi.importActual<typeof import("recharts")>("recharts");
  return {
    ...actual,
    ResponsiveContainer: ({ children }: { children: ReactElement }) =>
      isValidElement(children)
        ? cloneElement(children, { width: 900, height: 300 } as Partial<typeof children.props>)
        : children,
  };
});

function dateAt(i: number): string {
  return new Date(Date.UTC(2020, 0, 1 + i)).toISOString().slice(0, 10);
}

function nights(n: number): SleepRecord[] {
  return Array.from({ length: n }, (_, i) => ({
    id: i + 1,
    date: dateAt(i),
    source: "withings",
    duration_total: 7 * 3600,
    duration_deep: 3600,
    duration_light: 4 * 3600,
    duration_rem: 3600,
    duration_awake: 1800,
  })) as SleepRecord[];
}

function walks(n: number): Activity[] {
  return Array.from({ length: n }, (_, i) => ({
    id: i + 1,
    date: dateAt(i),
    source: "withings",
    steps: 5000 + (i % 7) * 500,
  })) as Activity[];
}

const bars = (c: HTMLElement) => c.querySelectorAll(".recharts-bar-rectangle").length;

describe("histogrammes agrégés au-delà de 150 barres", () => {
  it("sommeil : une pile par semaine au-delà de 150 nuits, avec une note", async () => {
    const { container } = render(<SleepChart data={nights(400)} />);
    await waitFor(() => expect(bars(container)).toBeGreaterThan(0), { timeout: 8000 });
    expect(bars(container)).toBeLessThanOrEqual(4 * 150);
    expect(screen.getByText(/semaines affichées pour 400 nuits/)).toBeInTheDocument();
  });

  it("sommeil : une pile par mois sur dix ans", async () => {
    const { container } = render(<SleepChart data={nights(3650)} />);
    await waitFor(() => expect(bars(container)).toBeGreaterThan(0), { timeout: 20000 });
    expect(bars(container)).toBeLessThanOrEqual(4 * 150);
    expect(screen.getByText(/mois affichés pour 3650 nuits/)).toBeInTheDocument();
  }, 40000);

  it("sommeil : une pile par nuit sous le plafond, sans note", async () => {
    const { container } = render(<SleepChart data={nights(30)} />);
    await waitFor(() => expect(bars(container)).toBe(30 * 4), { timeout: 8000 });
    expect(screen.queryByText(/affichés pour|affichées pour/)).toBeNull();
  });

  it("activité : une barre par semaine au-delà de 150 jours, avec une note", async () => {
    const { container } = render(<ActivityChart data={walks(400)} />);
    await waitFor(() => expect(bars(container)).toBeGreaterThan(0), { timeout: 8000 });
    expect(bars(container)).toBeLessThanOrEqual(150);
    expect(screen.getByText(/semaines affichées pour 400 jours/)).toBeInTheDocument();
  });

  it("activité : une barre par jour sous le plafond, sans note", async () => {
    const { container } = render(<ActivityChart data={walks(30)} />);
    await waitFor(() => expect(bars(container)).toBe(30), { timeout: 8000 });
    expect(screen.queryByText(/affichés pour|affichées pour/)).toBeNull();
  });

  it("la note d'agrégation est lue, hors de toute image, et le libellé dit les moyennes", async () => {
    const { container } = render(<SleepChart data={nights(400)} />);
    await waitFor(() => expect(bars(container)).toBeGreaterThan(0), { timeout: 8000 });
    const figure = screen.getByRole("figure");
    expect(figure.getAttribute("aria-label")).toMatch(/moyennes par semaine/);
    expect(screen.queryByRole("img")).toBeNull();
    expect(screen.getByText(/semaines affichées pour 400 nuits/)).toBeInTheDocument();
  });
});
