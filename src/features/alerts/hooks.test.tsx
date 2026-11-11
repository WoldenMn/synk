import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import {
  useAlerts,
  useAlertEvents,
  useAddAlert,
  useDeleteAlert,
  useCheckAlerts,
  useMarkAlertsSeen,
} from "./hooks";
import * as api from "./api";
import { reportError } from "@/lib/reportError";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));
vi.mock("./api", () => ({
  getAlerts: vi.fn(),
  addAlert: vi.fn(),
  deleteAlert: vi.fn(),
  getAlertEvents: vi.fn(),
  markAlertsSeen: vi.fn(),
  checkAlerts: vi.fn(),
}));
vi.mock("@/lib/reportError", () => ({
  reportError: vi.fn(),
}));

const mockApi = vi.mocked(api);
const mockReportError = reportError as unknown as ReturnType<typeof vi.fn>;

function makeWrapper(client: QueryClient) {
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
}

function newQc() {
  return new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
}

describe("useAlerts / useAlertEvents", () => {
  beforeEach(() => vi.clearAllMocks());

  it("recupere les alertes via getAlerts", async () => {
    const fake = [
      {
        id: 1,
        metric_key: "weight",
        condition: "above",
        threshold: 90,
        label: "Poids",
        enabled: true,
        created_at: "2026-04-20T08:00:00Z",
      },
    ];
    mockApi.getAlerts.mockResolvedValue(fake);

    const { result } = renderHook(() => useAlerts(), { wrapper: makeWrapper(newQc()) });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockApi.getAlerts).toHaveBeenCalled();
    expect(result.current.data).toEqual(fake);
  });

  it("recupere les événements via getAlertEvents (queryKey alerts/events)", async () => {
    mockApi.getAlertEvents.mockResolvedValue([]);

    const { result } = renderHook(() => useAlertEvents(), { wrapper: makeWrapper(newQc()) });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockApi.getAlertEvents).toHaveBeenCalled();
  });
});

describe("alert mutations", () => {
  beforeEach(() => vi.clearAllMocks());

  it("useAddAlert invalide ['alerts'] apres succès", async () => {
    mockApi.addAlert.mockResolvedValue(1);
    const qc = newQc();
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");
    const { result } = renderHook(() => useAddAlert(), { wrapper: makeWrapper(qc) });

    const input = { metric_key: "weight", condition: "above", threshold: 90, label: "Poids" };
    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockApi.addAlert).toHaveBeenCalledWith(input);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["alerts"] });
  });

  it("useDeleteAlert invalide ['alerts'] apres succès", async () => {
    mockApi.deleteAlert.mockResolvedValue(undefined);
    const qc = newQc();
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");
    const { result } = renderHook(() => useDeleteAlert(), { wrapper: makeWrapper(qc) });

    await act(async () => {
      await result.current.mutateAsync(3);
    });

    expect(mockApi.deleteAlert).toHaveBeenCalledWith(3);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["alerts"] });
  });

  it("useCheckAlerts invalide ['alerts', 'events'] apres succès", async () => {
    mockApi.checkAlerts.mockResolvedValue([]);
    const qc = newQc();
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");
    const { result } = renderHook(() => useCheckAlerts(), { wrapper: makeWrapper(qc) });

    await act(async () => {
      await result.current.mutateAsync();
    });

    expect(mockApi.checkAlerts).toHaveBeenCalled();
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["alerts", "events"] });
  });

  it("useMarkAlertsSeen invalide ['alerts', 'events'] apres succès", async () => {
    mockApi.markAlertsSeen.mockResolvedValue(undefined);
    const qc = newQc();
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");
    const { result } = renderHook(() => useMarkAlertsSeen(), { wrapper: makeWrapper(qc) });

    await act(async () => {
      await result.current.mutateAsync();
    });

    expect(mockApi.markAlertsSeen).toHaveBeenCalled();
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["alerts", "events"] });
  });

  it("appelle reportError avec context alerts si une mutation rejette", async () => {
    mockApi.deleteAlert.mockRejectedValue(new Error("db error"));
    const { result } = renderHook(() => useDeleteAlert(), { wrapper: makeWrapper(newQc()) });

    await act(async () => {
      await result.current.mutateAsync(3).catch(() => undefined);
    });

    await waitFor(() => expect(mockReportError).toHaveBeenCalled());
    expect(mockReportError).toHaveBeenCalledWith(expect.any(Error), {
      source: "mutation",
      context: "alerts",
    });
  });
});
