import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useErrorLog, usePurgeErrorLog } from "./hooks";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function makeWrapper(client: QueryClient) {
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return Wrapper;
}

describe("useErrorLog", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("recupere le journal via get_error_log sans filtre", async () => {
    const fake = [
      {
        id: 1,
        occurred_at: "2026-06-19 10:00:00",
        source: "sync",
        code: "AUTH",
        message: "x",
        context: null,
      },
    ];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useErrorLog(), { wrapper: makeWrapper(qc) });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_error_log", { limit: 200, codeFilter: null });
    expect(result.current.data).toEqual(fake);
  });

  it("passe le filtre de code et l'inclut dans la queryKey", async () => {
    mockInvoke.mockResolvedValue([]);

    const { result } = renderHook(() => useErrorLog("NETWORK"), { wrapper: makeWrapper(qc) });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_error_log", { limit: 200, codeFilter: "NETWORK" });
    expect(qc.getQueryData(["errorLog", "NETWORK"])).toEqual([]);
  });
});

describe("usePurgeErrorLog", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("invalide ['errorLog'] apres succès", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => usePurgeErrorLog(), { wrapper: makeWrapper(qc) });

    await act(async () => {
      await result.current.mutateAsync();
    });

    expect(mockInvoke).toHaveBeenCalledWith("purge_error_log");
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["errorLog"] });
  });
});
