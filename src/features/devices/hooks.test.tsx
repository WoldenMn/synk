import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useDevices } from "./hooks";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function wrapper({ children }: { children: React.ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("useDevices", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("retourne la liste des appareils Withings via invoke get_withings_devices", async () => {
    const fake = [{ device_id: "abc", model: "Body+", battery: 80 }];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useDevices(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(fake);
    expect(mockInvoke).toHaveBeenCalledWith("get_withings_devices");
  });
});
