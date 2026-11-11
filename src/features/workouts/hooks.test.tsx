import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useWorkoutsData, useLatestWorkout, useAddWorkout, useDeleteWorkout } from "./hooks";

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

describe("useWorkoutsData", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("recupere via get_workouts avec from/to", async () => {
    const fake = [
      { id: 1, start_at: "2026-04-20T07:00:00Z", category: "Course", duration_seconds: 3600 },
    ];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useWorkoutsData("2026-04-01", "2026-04-30"), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_workouts", {
      from: "2026-04-01",
      to: "2026-04-30",
    });
    expect(result.current.data).toEqual(fake);
  });
});

describe("useLatestWorkout", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("retourne le dernier entraînement via get_latest_workout", async () => {
    const fake = { id: 11, start_at: "2026-04-26T08:00:00Z", category: "Vélo" };
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useLatestWorkout(), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_latest_workout");
    expect(result.current.data).toEqual(fake);
  });
});

describe("useAddWorkout", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("invalide ['workouts'] apres succès", async () => {
    mockInvoke.mockResolvedValue(55);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useAddWorkout(), {
      wrapper: makeWrapper(qc),
    });

    const data = {
      date: "2026-04-29",
      category_label: "Course",
      duration: 1800,
      distance: 5000,
      steps: 6000,
      calories: 350,
      hr_average: 145,
    };
    await act(async () => {
      await result.current.mutateAsync(data);
    });

    expect(mockInvoke).toHaveBeenCalledWith("add_manual_workout", { data });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["workouts"] });
  });
});

describe("useDeleteWorkout", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("supprime via delete_workout et invalide ['workouts']", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useDeleteWorkout(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync(8);
    });

    expect(mockInvoke).toHaveBeenCalledWith("delete_workout", { id: 8 });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["workouts"] });
  });
});
