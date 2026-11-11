import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useMeasurementTypes, useAddManualMeasurement, useDeleteManualMeasurement } from "./hooks";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function makeWrapper(client: QueryClient) {
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return Wrapper;
}

describe("useMeasurementTypes", () => {
  let qc: QueryClient;

  beforeEach(() => {
    qc = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0 } },
    });
    mockInvoke.mockReset();
  });

  it("retourne la liste via get_measurement_types", async () => {
    const fake = [
      {
        id: 1,
        slug: "tour_taille",
        label: "Tour de taille",
        unit: "cm",
        category: "body",
        sort_order: 0,
        is_builtin: 1,
        target_value: null,
        target_direction: null,
      },
    ];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useMeasurementTypes(), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_measurement_types");
    expect(result.current.data).toEqual(fake);
  });

  it("useAddManualMeasurement invalide ['manualMeasurements'] et ['measurementStats']", async () => {
    mockInvoke.mockResolvedValue(123);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useAddManualMeasurement(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync({
        measurement_type_id: 1,
        measured_at: "2026-04-29T08:00:00Z",
        value: 85.5,
        notes: null,
      });
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["manualMeasurements"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["measurementStats"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["clothingInputs"] });
  });

  it("useDeleteManualMeasurement invalide ['clothingInputs']", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useDeleteManualMeasurement(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync(1);
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["clothingInputs"] });
  });
});
