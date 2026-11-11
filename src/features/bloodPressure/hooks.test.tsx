import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import {
  useBloodPressureData,
  useLatestBloodPressure,
  useAddBloodPressure,
  useDeleteBloodPressure,
} from "./hooks";

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

describe("useBloodPressureData", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("recupere les mesures via get_blood_pressure avec from/to", async () => {
    const fake = [
      {
        id: 1,
        measured_at: "2026-04-20T08:00:00Z",
        source: "manual",
        systolic: 120,
        diastolic: 80,
        heart_rate: 65,
      },
    ];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useBloodPressureData("2026-04-01", "2026-04-30"), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_blood_pressure", {
      from: "2026-04-01",
      to: "2026-04-30",
    });
    expect(result.current.data).toEqual(fake);
  });

  it("expose une erreur si invoke échoué", async () => {
    mockInvoke.mockRejectedValue(new Error("db error"));

    const { result } = renderHook(() => useBloodPressureData("2026-04-01", "2026-04-30"), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
  });
});

describe("useLatestBloodPressure", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("retourne la dernière mesure via get_latest_blood_pressure", async () => {
    const fake = {
      id: 5,
      measured_at: "2026-04-25T08:00:00Z",
      source: "withings",
      systolic: 118,
      diastolic: 78,
      heart_rate: 60,
    };
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useLatestBloodPressure(), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_latest_blood_pressure");
    expect(result.current.data).toEqual(fake);
  });
});

describe("useAddBloodPressure", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("invalide ['bloodPressure'] apres succès", async () => {
    mockInvoke.mockResolvedValue(42);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useAddBloodPressure(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync({
        measured_at: "2026-04-29T08:00:00Z",
        systolic: 120,
        diastolic: 80,
        heart_rate: 65,
      });
    });

    expect(mockInvoke).toHaveBeenCalledWith("add_manual_blood_pressure", {
      data: {
        measured_at: "2026-04-29T08:00:00Z",
        systolic: 120,
        diastolic: 80,
        heart_rate: 65,
      },
    });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["bloodPressure"] });
  });
});

describe("useDeleteBloodPressure", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("supprime via delete_blood_pressure et invalide ['bloodPressure']", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useDeleteBloodPressure(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync(7);
    });

    expect(mockInvoke).toHaveBeenCalledWith("delete_blood_pressure", { id: 7 });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["bloodPressure"] });
  });
});
