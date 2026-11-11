import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import {
  useMetricDefinitions,
  useMetricSeries,
  useMetricLatest,
  useMetricSecondary,
} from "./hooks";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function wrapper({ children }: { children: React.ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("useMetricDefinitions", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("retourne toutes les définitions sans filtre de catégorie", async () => {
    const fake = [
      {
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
      },
    ];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useMetricDefinitions(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(fake);
    expect(mockInvoke).toHaveBeenCalledWith("get_metric_definitions", { category: undefined });
  });

  it("filtre par catégorie quand fournie", async () => {
    mockInvoke.mockResolvedValue([]);

    const { result } = renderHook(() => useMetricDefinitions("body_composition"), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_metric_definitions", {
      category: "body_composition",
    });
  });
});

describe("useMetricSeries", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("recupere une serie avec slug + range de dates", async () => {
    const fake = [
      { measured_at: "2026-04-20T08:00:00Z", value: 72.4, source: "withings" },
      { measured_at: "2026-04-21T08:00:00Z", value: 72.1, source: "withings" },
    ];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useMetricSeries("weight", "2026-04-01", "2026-04-30"), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(fake);
    expect(mockInvoke).toHaveBeenCalledWith("get_metric_series", {
      slug: "weight",
      from: "2026-04-01",
      to: "2026-04-30",
    });
  });

  it("ne fetch pas si slug vide", async () => {
    mockInvoke.mockResolvedValue([]);

    const { result } = renderHook(() => useMetricSeries("", "2026-04-01", "2026-04-30"), {
      wrapper,
    });

    expect(result.current.isLoading).toBe(false);
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});

describe("useMetricSecondary", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("lit les points des sources non retenues sur la période", async () => {
    const fake = [{ measured_at: "2026-04-20T09:00:00Z", value: 72.0, source: "manual" }];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useMetricSecondary("weight", "2026-04-01", "2026-04-30"), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(fake);
    expect(mockInvoke).toHaveBeenCalledWith("get_metric_secondary_points", {
      slug: "weight",
      from: "2026-04-01",
      to: "2026-04-30",
    });
  });

  it("ne lit rien quand la lecture est désactivée", () => {
    renderHook(() => useMetricSecondary("weight", "2026-04-01", "2026-04-30", false), { wrapper });
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});

describe("useMetricLatest", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("retourne le dernier point", async () => {
    const fake = { measured_at: "2026-04-25T08:00:00Z", value: 71.8, source: "withings" };
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useMetricLatest("weight"), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(fake);
    expect(mockInvoke).toHaveBeenCalledWith("get_metric_latest", { slug: "weight" });
  });
});
