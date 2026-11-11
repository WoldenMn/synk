import { describe, expect, it, vi } from "vitest";
import { cloneElement, isValidElement } from "react";
import type { ReactElement } from "react";
import { render, screen, waitFor } from "@testing-library/react";
import { WeightChart } from "./WeightChart";
import type { BodyMeasure } from "@/features/bodyMeasures/types";

vi.mock("recharts", async () => {
  const actual = await vi.importActual<typeof import("recharts")>("recharts");
  return {
    ...actual,
    ResponsiveContainer: ({ children }: { children: ReactElement }) =>
      isValidElement(children)
        ? cloneElement(children, { width: 800, height: 300 } as Partial<typeof children.props>)
        : children,
  };
});

function measures(n: number): BodyMeasure[] {
  return Array.from({ length: n }, (_, i) => ({
    id: i + 1,
    measured_at: new Date(Date.UTC(2020, 0, 1 + i)).toISOString(),
    source: "manual",
    weight: 70 + (i % 5),
    fat_ratio: null,
    fat_mass: null,
    fat_free_mass: null,
    muscle_mass: null,
    bone_mass: null,
    hydration: null,
    bmi: null,
  }));
}

describe("graphiques décimés", () => {
  it("annonce le nombre de points affichés au-delà de 500", () => {
    render(<WeightChart data={measures(2300)} />);
    expect(screen.getByText(/500 points affichés sur 2300/)).toBeInTheDocument();
  });

  it("n'annonce rien sous le plafond", () => {
    render(<WeightChart data={measures(120)} />);
    expect(screen.queryByText(/points affichés sur/)).toBeNull();
  });

  it("dessine un cercle par mesure sur une courte série, aucun sur une longue", async () => {
    const short = render(<WeightChart data={measures(30)} />);
    await waitFor(
      () =>
        expect(short.container.querySelectorAll("circle.recharts-dot").length).toBeGreaterThan(0),
      { timeout: 4000 }
    );
    short.unmount();
    const long = render(<WeightChart data={measures(120)} />);
    await new Promise((r) => setTimeout(r, 2500));
    const withoutDots = long.container.querySelectorAll("circle.recharts-dot").length;
    expect(withoutDots).toBe(0);
  });
});
