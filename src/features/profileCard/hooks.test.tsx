import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invokeMock(...args) }));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { useProfileCard } from "./hooks";

function wrapper({ children }: { children: ReactNode }) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={qc}>{children}</QueryClientProvider>;
}

describe("hooks profileCard", () => {
  beforeEach(() => invokeMock.mockReset());

  it("charge la carte via get_profile_card", async () => {
    invokeMock.mockResolvedValue({
      body: {
        age: 36,
        sex: "male",
        height_cm: 178,
        weight_kg: 74.5,
        bmi: 23.5,
        fat_ratio: null,
        muscle_mass: null,
        hydration: null,
        measured_at: "2026-08-01",
      },
      key_measurements: [],
      health: { active_treatments: [] },
      sizes: [],
      records: [],
    });
    const { result } = renderHook(() => useProfileCard(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invokeMock).toHaveBeenCalledWith("get_profile_card");
    expect(result.current.data?.body.age).toBe(36);
  });
});
