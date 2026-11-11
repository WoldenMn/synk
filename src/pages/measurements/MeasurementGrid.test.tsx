import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { MeasurementGrid } from "./MeasurementGrid";
import type { MeasurementType } from "@/features/manualMeasurements/types";

const mockInvoke = vi.mocked(invoke);

function type(id: number, label: string): MeasurementType {
  return {
    id,
    slug: `t${id}`,
    label,
    unit: "cm",
    category: "body",
    sort_order: id,
    is_builtin: 1,
    target_value: null,
    target_direction: null,
  };
}

describe("MeasurementGrid", () => {
  it("lit toutes les cartes en une seule commande", async () => {
    mockInvoke.mockReset();
    mockInvoke.mockImplementation((cmd: string) =>
      Promise.resolve(
        cmd === "get_measurement_overview"
          ? [
              {
                type_id: 1,
                values: [80, 81],
                stats: {
                  latest_value: 81,
                  latest_date: "2026-09-20",
                  min_value: 80,
                  max_value: 81,
                  avg_value: 80.5,
                  count: 2,
                  prev_value: null,
                  percent_change: null,
                },
              },
            ]
          : []
      )
    );
    render(
      <QueryClientProvider client={new QueryClient()}>
        <MeasurementGrid
          types={[type(1, "Bras"), type(2, "Mollet"), type(3, "Cou")]}
          isLoading={false}
          onSelect={vi.fn()}
          onAddEntry={vi.fn()}
          onManage={vi.fn()}
        />
      </QueryClientProvider>
    );

    expect(await screen.findByText("81,0")).toBeInTheDocument();
    const commands = mockInvoke.mock.calls.map((c) => c[0]);
    expect(commands.filter((c) => c === "get_measurement_overview")).toHaveLength(1);
    expect(commands).not.toContain("get_measurement_stats");
    expect(commands).not.toContain("get_manual_measurements");
  });
});
