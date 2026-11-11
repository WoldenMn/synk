import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import { useLabResults, useAddLabResult, useDeleteLabResult } from "./hooks";
import * as api from "./api";
import { reportError } from "@/lib/reportError";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));
vi.mock("./api", () => ({
  getLabResults: vi.fn(),
  addLabResult: vi.fn(),
  deleteLabResult: vi.fn(),
}));
vi.mock("@/lib/reportError", () => ({
  reportError: vi.fn(),
}));

const mockGetLabResults = api.getLabResults as unknown as ReturnType<typeof vi.fn>;
const mockAddLabResult = api.addLabResult as unknown as ReturnType<typeof vi.fn>;
const mockDeleteLabResult = api.deleteLabResult as unknown as ReturnType<typeof vi.fn>;
const mockReportError = reportError as unknown as ReturnType<typeof vi.fn>;

function makeWrapper(client: QueryClient) {
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
}

describe("useLabResults", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    vi.clearAllMocks();
  });

  it("recupere les résultats via getLabResults avec from/to", async () => {
    const fake = [
      {
        id: 1,
        date: "2026-04-20",
        test_name: "Glucose",
        value: 0.9,
        unit: "g/L",
        reference_min: 0.7,
        reference_max: 1.1,
        notes: null,
        created_at: "2026-04-20T08:00:00Z",
      },
    ];
    mockGetLabResults.mockResolvedValue(fake);

    const { result } = renderHook(() => useLabResults("2026-04-01", "2026-04-30"), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockGetLabResults).toHaveBeenCalledWith("2026-04-01", "2026-04-30");
    expect(result.current.data).toEqual(fake);
  });

  it("ne s'execute pas si from/to vides", () => {
    const { result } = renderHook(() => useLabResults("", ""), {
      wrapper: makeWrapper(qc),
    });
    expect(result.current.fetchStatus).toBe("idle");
    expect(mockGetLabResults).not.toHaveBeenCalled();
  });
});

describe("useAddLabResult", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    vi.clearAllMocks();
  });

  it("appelle addLabResult et invalide ['labs'] apres succès", async () => {
    mockAddLabResult.mockResolvedValue(42);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useAddLabResult(), { wrapper: makeWrapper(qc) });

    const input = {
      date: "2026-04-29",
      test_name: "HbA1c",
      value: 5.2,
      unit: "%",
      reference_min: null,
      reference_max: null,
      notes: null,
    };
    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockAddLabResult).toHaveBeenCalledWith(input);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["labs"] });
    expect(mockReportError).not.toHaveBeenCalled();
  });

  it("appelle reportError si addLabResult rejette", async () => {
    mockAddLabResult.mockRejectedValue(new Error("db error"));

    const { result } = renderHook(() => useAddLabResult(), { wrapper: makeWrapper(qc) });

    await act(async () => {
      await result.current
        .mutateAsync({
          date: "2026-04-29",
          test_name: "HbA1c",
          value: 5.2,
          unit: null,
          reference_min: null,
          reference_max: null,
          notes: null,
        })
        .catch(() => undefined);
    });

    await waitFor(() => expect(mockReportError).toHaveBeenCalled());
    expect(mockReportError).toHaveBeenCalledWith(expect.any(Error), {
      source: "mutation",
      context: "labs",
    });
  });
});

describe("useDeleteLabResult", () => {
  let qc: QueryClient;
  beforeEach(() => {
    qc = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
    vi.clearAllMocks();
  });

  it("supprime via deleteLabResult et invalide ['labs']", async () => {
    mockDeleteLabResult.mockResolvedValue(undefined);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useDeleteLabResult(), { wrapper: makeWrapper(qc) });

    await act(async () => {
      await result.current.mutateAsync(7);
    });

    expect(mockDeleteLabResult).toHaveBeenCalledWith(7);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["labs"] });
  });

  it("appelle reportError si deleteLabResult rejette", async () => {
    mockDeleteLabResult.mockRejectedValue(new Error("boom"));

    const { result } = renderHook(() => useDeleteLabResult(), { wrapper: makeWrapper(qc) });

    await act(async () => {
      await result.current.mutateAsync(7).catch(() => undefined);
    });

    await waitFor(() => expect(mockReportError).toHaveBeenCalled());
    expect(mockReportError).toHaveBeenCalledWith(expect.any(Error), {
      source: "mutation",
      context: "labs",
    });
  });
});
