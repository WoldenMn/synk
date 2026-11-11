import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useSleepData, useLatestSleep, useAddSleep, useDeleteSleep } from "./hooks";

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

describe("useSleepData", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("recupere les nuits via get_sleep_records avec from/to", async () => {
    const fake = [
      {
        id: 1,
        start_at: "2026-04-20T22:00:00Z",
        end_at: "2026-04-21T06:30:00Z",
        duration_total: 30600,
        sleep_score: 78,
      },
    ];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useSleepData("2026-04-01", "2026-04-30"), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_sleep_records", {
      from: "2026-04-01",
      to: "2026-04-30",
    });
    expect(result.current.data).toEqual(fake);
  });
});

describe("useLatestSleep", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("retourne la dernière nuit via get_latest_sleep", async () => {
    const fake = { id: 9, start_at: "2026-04-25T22:00:00Z", duration_total: 28800 };
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useLatestSleep(), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_latest_sleep");
    expect(result.current.data).toEqual(fake);
  });
});

describe("useAddSleep", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("invalide ['sleep'] apres succès", async () => {
    mockInvoke.mockResolvedValue(99);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useAddSleep(), {
      wrapper: makeWrapper(qc),
    });

    const data = {
      date: "2026-04-29",
      start_at: "2026-04-28T22:00:00Z",
      end_at: "2026-04-29T06:00:00Z",
      duration_total: 28800,
      sleep_score: 80,
    };
    await act(async () => {
      await result.current.mutateAsync(data);
    });

    expect(mockInvoke).toHaveBeenCalledWith("add_manual_sleep", { data });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["sleep"] });
  });
});

describe("useDeleteSleep", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("supprime via delete_sleep_record et invalide ['sleep']", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useDeleteSleep(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync(12);
    });

    expect(mockInvoke).toHaveBeenCalledWith("delete_sleep_record", { id: 12 });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["sleep"] });
  });
});
