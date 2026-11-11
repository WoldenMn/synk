import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { LabResult } from "@/features/labs/types";
import { LabAnalysesList } from "./LabAnalysesList";

const result = (id: number, value: number): LabResult => ({
  id,
  date: `2026-0${id}-01`,
  test_name: "Ferritine",
  value,
  unit: "ng/mL",
  reference_min: 30,
  reference_max: 300,
  notes: null,
  created_at: "2026-01-01T00:00:00Z",
});

describe("LabAnalysesList", () => {
  it("relie le bouton de l'accordéon à son panneau", () => {
    const groups = new Map([["Ferritine", [result(2, 20), result(1, 80)]]]);
    const props = {
      loading: false,
      groupedByTest: groups,
      setExpandedTest: vi.fn(),
      onDelete: vi.fn(),
    };
    const { rerender } = render(<LabAnalysesList {...props} expandedTest={null} />);
    const toggle = screen.getByRole("button", { name: /Ferritine/ });
    expect(toggle).toHaveAttribute("aria-expanded", "false");

    rerender(<LabAnalysesList {...props} expandedTest="Ferritine" />);
    const opened = screen.getByRole("button", { name: /Ferritine/ });
    expect(opened).toHaveAttribute("aria-expanded", "true");
    const panel = document.getElementById(opened.getAttribute("aria-controls") ?? "");
    expect(panel).not.toBeNull();
  });
});
