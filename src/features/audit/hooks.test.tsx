import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import {
  useAuditMetrics,
  usePurgeAuditLog,
  useAuditPurgeConfig,
  useSetAuditPurgeConfig,
} from "./hooks";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function makeWrapper(client: QueryClient) {
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return Wrapper;
}

describe("useAuditMetrics", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("recupere les métriques via get_audit_metrics", async () => {
    const fake = {
      total_count: 100,
      insert_count: 60,
      delete_count: 40,
      oldest_at: "2026-01-01 10:00:00",
      newest_at: "2026-05-07 15:00:00",
    };
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useAuditMetrics(), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_audit_metrics");
    expect(result.current.data).toEqual(fake);
  });
});

describe("usePurgeAuditLog", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("invalide ['audit_metrics'] et ['db_metrics'] apres succès", async () => {
    mockInvoke.mockResolvedValue(42);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => usePurgeAuditLog(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync(90);
    });

    expect(mockInvoke).toHaveBeenCalledWith("purge_audit_log", { olderThanDays: 90 });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["audit_metrics"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["db_metrics"] });
  });
});

describe("useAuditPurgeConfig", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("recupere la config via get_audit_purge_config", async () => {
    const fake = { retention_days: 180, max_entries: 10000, enabled: true };
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useAuditPurgeConfig(), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_audit_purge_config");
    expect(result.current.data).toEqual(fake);
  });
});

describe("useSetAuditPurgeConfig", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    mockInvoke.mockReset();
  });

  it("invalide ['audit_purge_config'] apres succès", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");
    const config = { retention_days: 90, max_entries: 5000, enabled: false };

    const { result } = renderHook(() => useSetAuditPurgeConfig(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync(config);
    });

    expect(mockInvoke).toHaveBeenCalledWith("set_audit_purge_config", { config });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["audit_purge_config"] });
  });
});
