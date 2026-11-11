import { describe, expect, it, vi } from "vitest";
import { cloneElement, isValidElement } from "react";
import type { ReactElement } from "react";
import { render, screen } from "@testing-library/react";
import { BodyCompositionChart } from "./BodyCompositionChart";
import type { BodyMeasure } from "@/features/bodyMeasures/types";

vi.mock("recharts", async () => {
  const actual = await vi.importActual<typeof import("recharts")>("recharts");
  return {
    ...actual,
    ResponsiveContainer: ({ children }: { children: ReactElement }) =>
      isValidElement(children)
        ? cloneElement(children, { width: 400, height: 240 } as Partial<typeof children.props>)
        : children,
  };
});

function row(id: number, measured_at: string, bone: number | null): BodyMeasure {
  return {
    id,
    measured_at,
    source: "withings",
    weight: 75,
    fat_ratio: 20,
    fat_mass: 15 + id,
    fat_free_mass: 60,
    muscle_mass: 55 + id,
    bone_mass: bone,
    hydration: null,
    bmi: null,
  };
}

describe("BodyCompositionChart", () => {
  it("trace une courbe par série, chacune avec son axe, au lieu d'un seul axe commun", () => {
    render(
      <BodyCompositionChart
        data={[
          row(1, "2026-01-01T08:00:00", 3.1),
          row(2, "2026-03-01T08:00:00", 3.2),
          row(3, "2026-03-19T08:00:00", 3.1),
        ]}
      />
    );
    expect(screen.getAllByRole("figure")).toHaveLength(3);
    expect(screen.getByRole("figure", { name: /masse muscle/ })).toBeInTheDocument();
    expect(screen.getByRole("figure", { name: /masse graisse/ })).toBeInTheDocument();
    expect(screen.getByRole("figure", { name: /masse os/ })).toBeInTheDocument();
  });

  it("n'affiche pas une série que l'appareil ne mesure pas", () => {
    render(
      <BodyCompositionChart
        data={[row(1, "2026-01-01T08:00:00", null), row(2, "2026-03-01T08:00:00", null)]}
      />
    );
    expect(screen.getAllByRole("figure")).toHaveLength(2);
  });
});
