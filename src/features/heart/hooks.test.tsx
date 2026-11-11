import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useHeartData, useLatestHeart } from "./hooks";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function wrapper({ children }: { children: React.ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("useHeartData", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("recupere les enregistrements cœur via get_heart_records", async () => {
    const fake = [{ id: 1, recorded_at: "2026-04-20T08:00:00Z", heart_rate: 65 }];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useHeartData("2026-04-01", "2026-04-30"), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(fake);
    expect(mockInvoke).toHaveBeenCalledWith("get_heart_records", {
      from: "2026-04-01",
      to: "2026-04-30",
    });
  });
});

describe("useLatestHeart", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("retourne le dernier enregistrement cœur via get_latest_heart_record", async () => {
    const fake = { id: 5, recorded_at: "2026-04-25T08:00:00Z", heart_rate: 62 };
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useLatestHeart(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(fake);
    expect(mockInvoke).toHaveBeenCalledWith("get_latest_heart_record");
  });
});
