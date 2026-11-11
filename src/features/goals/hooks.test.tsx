import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useGoals, useUpsertGoal } from "./hooks";

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

describe("useGoals", () => {
  let qc: QueryClient;

  beforeEach(() => {
    qc = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0 } },
    });
    mockInvoke.mockReset();
  });

  it("useGoals retourne la liste via invoke get_goals", async () => {
    const fake = [{ id: 1, metric_key: "weight", target_value: 75, target_direction: "decrease" }];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useGoals(), { wrapper: makeWrapper(qc) });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_goals");
    expect(result.current.data).toEqual(fake);
  });

  it("useUpsertGoal invalide ['goals'] apres succès", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useUpsertGoal(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync({
        metric_key: "weight",
        target_value: 75,
        direction: "decrease",
      });
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["goals"] });
  });
});
