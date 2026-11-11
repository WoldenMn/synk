import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invokeMock(...args) }));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { useBodySheet, useAddMeasurementSession } from "./hooks";

function wrapper({ children }: { children: ReactNode }) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={qc}>{children}</QueryClientProvider>;
}

describe("hooks bodyProfile", () => {
  beforeEach(() => invokeMock.mockReset());

  it("charge la fiche via get_body_sheet", async () => {
    invokeMock.mockResolvedValue([
      { slug: "waist", label: "Tour de taille", unit: "cm", value: 86, measured_at: "2026-07-31" },
    ]);
    const { result } = renderHook(() => useBodySheet(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invokeMock).toHaveBeenCalledWith("get_body_sheet");
    expect(result.current.data?.[0]?.value).toBe(86);
  });

  it("enregistre un relevé via add_measurement_session", async () => {
    invokeMock.mockResolvedValue(3);
    const { result } = renderHook(() => useAddMeasurementSession(), { wrapper });
    result.current.mutate({
      measuredAt: "2026-07-31",
      entries: [{ slug: "waist", value: 86 }],
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invokeMock).toHaveBeenCalledWith("add_measurement_session", {
      measuredAt: "2026-07-31",
      entries: [{ slug: "waist", value: 86 }],
    });
  });
});
