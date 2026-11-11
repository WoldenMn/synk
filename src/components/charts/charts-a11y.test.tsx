import { describe, it, expect, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import { expectNoAxeViolations } from "@/test-utils/axe";
import { cloneElement, isValidElement } from "react";
import type { ReactElement } from "react";
import { WeightChart } from "./WeightChart";
import { BloodPressureChart } from "./BloodPressureChart";
import { GenericMetricChart } from "./GenericMetricChart";
import type { BodyMeasure } from "@/features/bodyMeasures/types";
import type { BloodPressure } from "@/features/bloodPressure/types";
import type { MetricDefinition, MetricPoint } from "@/features/metrics/types";

// ResponsiveContainer s'appuie sur ResizeObserver (absent de jsdom) et rend 0x0
// sans layout. On le remplace par un passe-plat qui injecte une taille fixe dans
// le chart enfant pour qu'il produise son SVG de maniere deterministe. Le reste de
// recharts est conserve (accessibilityLayer inclus).
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

function bodyMeasure(measured_at: string, weight: number): BodyMeasure {
  return {
    id: 1,
    measured_at,
    source: "manual",
    weight,
    fat_ratio: null,
    fat_mass: null,
    fat_free_mass: null,
    muscle_mass: null,
    bone_mass: null,
    hydration: null,
    bmi: null,
  };
}

function bp(measured_at: string, systolic: number, diastolic: number): BloodPressure {
  return { id: 1, measured_at, source: "manual", systolic, diastolic, heart_rate: 60 };
}

const metricDef: MetricDefinition = {
  id: 1,
  slug: "weight",
  label: "Poids",
  unit: "kg",
  category: "body_composition",
  withings_type: 1,
  is_builtin: 1,
  sort_order: 0,
  decimals: 1,
  min_value: null,
  max_value: null,
};

function metricPoint(measured_at: string, value: number): MetricPoint {
  return { measured_at, value, source: "withings" };
}

describe("charts a11y", () => {
  it("WeightChart expose figure nommée + aria-label refletant min/max", () => {
    render(
      <WeightChart
        data={[
          bodyMeasure("2026-01-01T08:00:00", 70),
          bodyMeasure("2026-01-08T08:00:00", 72),
          bodyMeasure("2026-01-15T08:00:00", 75),
        ]}
      />
    );
    const img = screen.getByRole("figure");
    const label = img.getAttribute("aria-label") ?? "";
    expect(label).toContain("Courbe du poids");
    expect(label).toContain("3 points");
    expect(label).toContain("70");
    expect(label).toContain("75");
    expect(label).toContain("kg");
  });

  it("BloodPressureChart expose figure nommée + aria-label avec le nombre de mesures", () => {
    render(
      <BloodPressureChart
        data={[bp("2026-01-01T08:00:00", 120, 80), bp("2026-01-02T08:00:00", 130, 85)]}
      />
    );
    const img = screen.getByRole("figure");
    const label = img.getAttribute("aria-label") ?? "";
    expect(label).toContain("Tension artérielle");
    expect(label).toContain("2 mesures");
  });

  it("GenericMetricChart expose figure nommée + aria-label base sur la définition", () => {
    render(
      <GenericMetricChart
        definition={metricDef}
        data={[metricPoint("2026-01-01T08:00:00", 70), metricPoint("2026-01-15T08:00:00", 74)]}
      />
    );
    const img = screen.getByRole("figure");
    const label = img.getAttribute("aria-label") ?? "";
    expect(label).toContain("Poids");
    expect(label).toContain("2 points");
    expect(label).toContain("kg");
  });

  it("GenericMetricChart nomme les sources dont les points restent en retrait", () => {
    render(
      <GenericMetricChart
        definition={metricDef}
        data={[metricPoint("2026-01-01T08:00:00", 70), metricPoint("2026-01-15T08:00:00", 74)]}
        secondary={[{ measured_at: "2026-01-15T09:00:00", value: 73, source: "manual" }]}
      />
    );
    expect(screen.getByText(/Points en retrait : Saisie manuelle/)).toBeInTheDocument();
    expect(screen.getByRole("figure").getAttribute("aria-label")).toContain(
      "1 point d'une autre source"
    );
  });

  it("GenericMetricChart ne parle pas d'autres sources quand il n'y en a pas", () => {
    render(
      <GenericMetricChart
        definition={metricDef}
        data={[metricPoint("2026-01-01T08:00:00", 70), metricPoint("2026-01-15T08:00:00", 74)]}
        secondary={[]}
      />
    );
    expect(screen.queryByText(/Points en retrait/)).not.toBeInTheDocument();
  });

  it("n'a aucune violation a11y critique/serieuse (WeightChart)", async () => {
    const { container } = render(
      <WeightChart
        data={[bodyMeasure("2026-01-01T08:00:00", 70), bodyMeasure("2026-01-15T08:00:00", 75)]}
      />
    );
    await expectNoAxeViolations(container);
  });

  it("le aria-label reflete une serie a valeur unique", () => {
    render(<WeightChart data={[bodyMeasure("2026-01-01T08:00:00", 80)]} />);
    const img = screen.getByRole("figure");
    const label = img.getAttribute("aria-label") ?? "";
    expect(label).toContain("1 point");
    expect(label).toContain("valeur 80 kg");
    expect(within(img).queryByRole("figure")).toBeNull();
  });
});
