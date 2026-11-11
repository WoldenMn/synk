import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useDbMetrics } from "./hooks";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function makeWrapper(client: QueryClient) {
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return Wrapper;
}

describe("useDbMetrics", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("recupere les métriques DB via get_db_metrics", async () => {
    const fake = {
      db_size_bytes: 12345678,
      audit_log_count: 500,
      body_measures_count: 100,
      blood_pressure_count: 50,
      sleep_count: 30,
      activity_count: 200,
      workouts_count: 20,
    };
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useDbMetrics(), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_db_metrics");
    expect(result.current.data).toEqual(fake);
  });
});
