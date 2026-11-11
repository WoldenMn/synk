import { describe, expect, it, vi } from "vitest";
import { cloneElement, isValidElement } from "react";
import type { ReactElement } from "react";
import { render, screen } from "@testing-library/react";
import { CorrelationChart } from "./CorrelationChart";
import type { MetricOption } from "../helpers";

vi.mock("recharts", async () => {
  const actual = await vi.importActual<typeof import("recharts")>("recharts");
  return {
    ...actual,
    ResponsiveContainer: ({ children }: { children: ReactElement }) =>
      isValidElement(children)
        ? cloneElement(children, { width: 900, height: 380 } as Partial<typeof children.props>)
        : children,
  };
});

const m1: MetricOption = {
  value: "weight",
  label: "Poids",
  unit: "kg",
  source: "body",
  field: "weight",
};
const m2: MetricOption = {
  value: "steps",
  label: "Pas",
  unit: "pas",
  source: "activity",
  field: "steps",
};

function points(n: number) {
  return Array.from({ length: n }, (_, i) => ({
    date: new Date(Date.UTC(2020, 0, 1 + i)).toISOString().slice(0, 10),
    v1: 70 + (i % 5),
    v2: 6000 + (i % 9) * 100,
  }));
}

function renderChart(n: number) {
  return render(
    <CorrelationChart
      m1={m1}
      m2={m2}
      chartData={points(n)}
      isLoading={false}
      color1="#6366f1"
      color2="#22c55e"
    />
  );
}

describe("CorrelationChart décimé", () => {
  it("annonce le nombre de points affichés au-delà de 500", () => {
    renderChart(2300);
    expect(screen.getByText(/500 points affichés sur 2300/)).toBeInTheDocument();
  });

  it("n'annonce rien sous le plafond", () => {
    renderChart(120);
    expect(screen.queryByText(/points affichés sur/)).toBeNull();
  });

  it("peint graduations et titres d'axe aux couleurs du thème, pas à celle de la série", () => {
    const { container } = renderChart(40);
    const texts = [...container.querySelectorAll(".recharts-cartesian-axis text, .recharts-label")];
    expect(texts.length).toBeGreaterThanOrEqual(2);
    const painted = texts.map(
      (t) => `${t.getAttribute("fill") ?? ""} ${t.getAttribute("style") ?? ""}`
    );
    for (const paint of painted) expect(paint).toContain("hsl(var(--muted-foreground))");
    for (const paint of painted) expect(paint).not.toMatch(/#6366f1|#22c55e|series/);
  });

  it("écrit les noms de la légende à la couleur du texte, pas à celle des séries", () => {
    const { container } = renderChart(40);
    const names = [...container.querySelectorAll(".recharts-legend-item-text")];
    expect(names.map((n) => n.textContent)).toEqual(["Poids", "Pas"]);
    for (const name of names) expect(name.querySelector("span.text-foreground")).not.toBeNull();
  });

  it("espace les graduations du temps selon les jours écoulés, pas selon le rang des points", () => {
    const rows = ["2020-01-01", "2020-01-02", "2020-01-31"].map((date, i) => ({
      date,
      v1: 70 + i,
      v2: 6000 + i,
    }));
    const { container } = render(
      <CorrelationChart
        m1={m1}
        m2={m2}
        chartData={rows}
        isLoading={false}
        color1="#6366f1"
        color2="#22c55e"
      />
    );
    const xs = [...container.querySelectorAll(".recharts-xAxis .recharts-cartesian-axis-tick-line")]
      .map((line) => Number(line.getAttribute("x1")))
      .sort((a, b) => a - b);
    expect(xs).toHaveLength(3);
    const [a = 0, b = 0, c = 0] = xs;
    expect((c - b) / (b - a)).toBeCloseTo(29, 0);
  });
});
