import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { LabResult } from "@/features/labs/types";
import { StatusDot, TrendArrow } from "./helpers";

const lab = (id: number, value: number): LabResult => ({
  id,
  date: "2026-01-01",
  test_name: "Ferritine",
  value,
  unit: null,
  reference_min: 30,
  reference_max: 300,
  notes: null,
  created_at: "2026-01-01T00:00:00Z",
});

describe("statut labo", () => {
  it("dit le statut en mots", () => {
    const { rerender } = render(<StatusDot inRange={false} />);
    expect(screen.getByText("hors référence")).toBeInTheDocument();
    rerender(<StatusDot inRange={true} />);
    expect(screen.getByText("dans la référence")).toBeInTheDocument();
    rerender(<StatusDot inRange={null} />);
    expect(screen.getByText("sans référence")).toBeInTheDocument();
  });

  it("dit le statut par la forme autant que par la couleur", () => {
    const { container, rerender } = render(<StatusDot inRange={true} />);
    const ok = container.querySelector("svg")?.outerHTML;
    rerender(<StatusDot inRange={false} />);
    const out = container.querySelector("svg")?.outerHTML;
    expect(ok).toBeDefined();
    expect(out).toBeDefined();
    expect(ok).not.toEqual(out);
  });

  it("dit le sens de la tendance", () => {
    const { rerender } = render(<TrendArrow items={[lab(2, 90), lab(1, 80)]} />);
    expect(screen.getByText("en hausse")).toBeInTheDocument();
    rerender(<TrendArrow items={[lab(2, 70), lab(1, 80)]} />);
    expect(screen.getByText("en baisse")).toBeInTheDocument();
  });
});
