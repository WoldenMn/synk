import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useBodyMeasures } from "./hooks";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function wrapper({ children }: { children: React.ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("useBodyMeasures", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("retourne les mesures apres un fetch réussi", async () => {
    const fakeData = [{ id: 1, measured_at: "2026-04-23T08:00:00Z", weight: 72.5 }];
    mockInvoke.mockResolvedValue(fakeData);

    const { result } = renderHook(() => useBodyMeasures("2026-04-01", "2026-04-30"), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(fakeData);
    expect(mockInvoke).toHaveBeenCalledWith("get_body_measures", {
      from: "2026-04-01",
      to: "2026-04-30",
    });
  });

  it("expose une erreur en cas d'échec", async () => {
    mockInvoke.mockRejectedValue(new Error("db error"));

    const { result } = renderHook(() => useBodyMeasures("2026-04-01", "2026-04-30"), { wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
  });
});
