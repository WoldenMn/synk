import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useActivityData, useLatestActivity, useAddActivity, useDeleteActivity } from "./hooks";

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

describe("useActivityData", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("recupere via get_activity avec from/to", async () => {
    const fake = [{ id: 1, date: "2026-04-20", steps: 8500, calories: 2100 }];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useActivityData("2026-04-01", "2026-04-30"), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_activity", {
      from: "2026-04-01",
      to: "2026-04-30",
    });
    expect(result.current.data).toEqual(fake);
  });
});

describe("useLatestActivity", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("retourne la dernière activité via get_latest_activity", async () => {
    const fake = { id: 7, date: "2026-04-25", steps: 9200 };
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useLatestActivity(), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_latest_activity");
    expect(result.current.data).toEqual(fake);
  });
});

describe("useAddActivity", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("invalide ['activity'] apres succès", async () => {
    mockInvoke.mockResolvedValue(33);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useAddActivity(), {
      wrapper: makeWrapper(qc),
    });

    const data = {
      date: "2026-04-29",
      steps: 10000,
      distance: 8000,
      calories_active: 350,
      duration_active: 3600,
    };
    await act(async () => {
      await result.current.mutateAsync(data);
    });

    expect(mockInvoke).toHaveBeenCalledWith("add_manual_activity", { data });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["activity"] });
  });
});

describe("useDeleteActivity", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("supprime via delete_activity et invalide ['activity']", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useDeleteActivity(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync(3);
    });

    expect(mockInvoke).toHaveBeenCalledWith("delete_activity", { id: 3 });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["activity"] });
  });
});
