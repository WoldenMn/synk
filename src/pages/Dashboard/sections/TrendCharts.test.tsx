import { describe, expect, it, vi } from "vitest";
import type { ComponentProps } from "react";
import { render, screen } from "@testing-library/react";
import { TrendCharts } from "./TrendCharts";

vi.mock("@/features/goals/hooks", () => ({ useGoals: () => ({ data: [] }) }));
vi.mock("@/features/sources/hooks", () => ({ useAnySourceConnected: () => ({ data: true }) }));

function renderCharts(props: Partial<ComponentProps<typeof TrendCharts>>) {
  return render(
    <TrendCharts
      bodyMeasures={[]}
      bpData={[]}
      sleepData={[]}
      activityData={[]}
      isLoading={false}
      timeRange="30d"
      onShowAll={vi.fn()}
      {...props}
    />
  );
}

describe("TrendCharts", () => {
  it("n'annonce aucune période vide pendant le chargement", () => {
    renderCharts({
      isLoading: true,
      bodyMeasures: undefined,
      bpData: undefined,
      sleepData: undefined,
      activityData: undefined,
    });
    expect(screen.queryByText(/Aucune pesée/)).not.toBeInTheDocument();
  });

  it("propose l'historique complet dans les quatre cartes vides", () => {
    renderCharts({});
    expect(screen.getAllByRole("button", { name: "Afficher tout l'historique" })).toHaveLength(4);
  });
});
