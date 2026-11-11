import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useSyncLog, useClearSyncLog } from "./hooks";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function makeWrapper(client: QueryClient) {
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return Wrapper;
}

describe("useSyncLog", () => {
  let qc: QueryClient;

  beforeEach(() => {
    qc = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0 } },
    });
    mockInvoke.mockReset();
  });

  it("recupere les entrées via get_sync_log avec limit par defaut", async () => {
    const fake = [
      {
        id: 1,
        started_at: "2026-04-20T08:00:00Z",
        finished_at: null,
        data_type: "measures",
        duration_ms: 500,
        records_added: 10,
        status: "success",
        error: null,
      },
    ];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useSyncLog(), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_sync_log", { limit: 100 });
    expect(result.current.data).toEqual(fake);
  });

  it("useClearSyncLog invalide ['sync_log'] apres succès", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useClearSyncLog(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync();
    });

    expect(mockInvoke).toHaveBeenCalledWith("clear_sync_log");
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["sync_log"] });
  });
});
