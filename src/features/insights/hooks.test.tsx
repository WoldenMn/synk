import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useInsights, useGenerateInsights } from "./hooks";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function makeWrapper(client: QueryClient) {
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return Wrapper;
}

describe("useInsights", () => {
  let qc: QueryClient;

  beforeEach(() => {
    qc = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0 } },
    });
    mockInvoke.mockReset();
  });

  it("recupere les insights via get_insights avec limit", async () => {
    const fake = [{ id: 1, kind: "trend", severity: "info", title: "Poids stable", message: "OK" }];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useInsights(10), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_insights", { limit: 10 });
    expect(result.current.data).toEqual(fake);
  });

  it("useGenerateInsights invalide ['insights'] apres succès", async () => {
    mockInvoke.mockResolvedValue(3);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useGenerateInsights(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync();
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["insights"] });
  });
});
