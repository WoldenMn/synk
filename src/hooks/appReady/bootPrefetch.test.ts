import { describe, it, expect, vi, beforeEach } from "vitest";
import { QueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { prefetchDashboard } from "./bootPrefetch";
import { rangeBounds } from "@/hooks/useTimeRange";

describe("prefetchDashboard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("prefetches the dashboard cards in one read, under the key the page uses", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => Promise.resolve({ cmd }));
    const qc = new QueryClient({
      defaultOptions: { queries: { staleTime: 1000 * 60 } },
    });

    await prefetchDashboard(qc);

    const { from, to } = rangeBounds("30d");
    const keys = qc
      .getQueryCache()
      .getAll()
      .map((q) => JSON.stringify(q.queryKey));
    expect(keys).toEqual([JSON.stringify(["dashboardCards", from, to])]);
    expect(vi.mocked(invoke)).toHaveBeenCalledTimes(1);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_dashboard_cards", { from, to });
  });

  it("does not reject when the read fails: prefetchQuery keeps the error in the cache", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("DB locked"));
    const qc = new QueryClient();

    await expect(prefetchDashboard(qc)).resolves.toBeUndefined();

    const { from, to } = rangeBounds("30d");
    expect(qc.getQueryState(["dashboardCards", from, to])?.status).toBe("error");
  });
});
