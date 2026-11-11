import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import {
  useMedications,
  useMedicationLogs,
  useAddMedication,
  useUpdateMedication,
  useDeleteMedication,
  useLogMedication,
} from "./hooks";
import * as api from "./api";
import { reportError } from "@/lib/reportError";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));
vi.mock("./api", () => ({
  getMedications: vi.fn(),
  addMedication: vi.fn(),
  updateMedication: vi.fn(),
  deleteMedication: vi.fn(),
  logMedicationTaken: vi.fn(),
  getMedicationLogs: vi.fn(),
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

describe("useMedications", () => {
  beforeEach(() => vi.clearAllMocks());

  it("recupere les médicaments via getMedications", async () => {
    const fake = [
      {
        id: 1,
        name: "Aspirine",
        dosage: null,
        frequency: null,
        start_date: null,
        end_date: null,
        notes: null,
        active: true,
        created_at: "2026-04-20T08:00:00Z",
      },
    ];
    mockApi.getMedications.mockResolvedValue(fake);

    const { result } = renderHook(() => useMedications(), { wrapper: makeWrapper(newQc()) });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockApi.getMedications).toHaveBeenCalled();
    expect(result.current.data).toEqual(fake);
  });
});

describe("useMedicationLogs", () => {
  beforeEach(() => vi.clearAllMocks());

  it("recupere les logs via getMedicationLogs quand medicationId > 0", async () => {
    mockApi.getMedicationLogs.mockResolvedValue([]);

    const { result } = renderHook(() => useMedicationLogs(3, "2026-04-01", "2026-04-30"), {
      wrapper: makeWrapper(newQc()),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockApi.getMedicationLogs).toHaveBeenCalledWith(3, "2026-04-01", "2026-04-30");
  });

  it("ne s'execute pas si medicationId <= 0", () => {
    const { result } = renderHook(() => useMedicationLogs(0, "2026-04-01", "2026-04-30"), {
      wrapper: makeWrapper(newQc()),
    });
    expect(result.current.fetchStatus).toBe("idle");
    expect(mockApi.getMedicationLogs).not.toHaveBeenCalled();
  });
});

describe("medication mutations", () => {
  beforeEach(() => vi.clearAllMocks());

  it("useAddMedication invalide ['medications'] apres succès", async () => {
    mockApi.addMedication.mockResolvedValue(1);
    const qc = newQc();
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");
    const { result } = renderHook(() => useAddMedication(), { wrapper: makeWrapper(qc) });

    const input = {
      name: "Vitamine D",
      dosage: null,
      frequency: null,
      start_date: null,
      end_date: null,
      notes: null,
    };
    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockApi.addMedication).toHaveBeenCalledWith(input);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["medications"] });
  });

  it("useUpdateMedication invalide ['medications'] apres succès", async () => {
    mockApi.updateMedication.mockResolvedValue(undefined);
    const qc = newQc();
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");
    const { result } = renderHook(() => useUpdateMedication(), { wrapper: makeWrapper(qc) });

    const input = {
      id: 1,
      name: "Vitamine D",
      dosage: null,
      frequency: null,
      start_date: null,
      end_date: null,
      notes: null,
      active: false,
    };
    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockApi.updateMedication).toHaveBeenCalledWith(input);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["medications"] });
  });

  it("useDeleteMedication invalide ['medications'] apres succès", async () => {
    mockApi.deleteMedication.mockResolvedValue(undefined);
    const qc = newQc();
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");
    const { result } = renderHook(() => useDeleteMedication(), { wrapper: makeWrapper(qc) });

    await act(async () => {
      await result.current.mutateAsync(5);
    });

    expect(mockApi.deleteMedication).toHaveBeenCalledWith(5);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["medications"] });
  });

  it("useLogMedication invalide ['medications'] apres succès", async () => {
    mockApi.logMedicationTaken.mockResolvedValue(1);
    const qc = newQc();
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");
    const { result } = renderHook(() => useLogMedication(), { wrapper: makeWrapper(qc) });

    const input = {
      medication_id: 1,
      taken_at: "2026-04-29T08:00:00Z",
      skipped: false,
      notes: null,
    };
    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockApi.logMedicationTaken).toHaveBeenCalledWith(input);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["medications"] });
  });

  it("appelle reportError avec context medications si une mutation rejette", async () => {
    mockApi.deleteMedication.mockRejectedValue(new Error("db error"));
    const { result } = renderHook(() => useDeleteMedication(), { wrapper: makeWrapper(newQc()) });

    await act(async () => {
      await result.current.mutateAsync(5).catch(() => undefined);
    });

    await waitFor(() => expect(mockReportError).toHaveBeenCalled());
    expect(mockReportError).toHaveBeenCalledWith(expect.any(Error), {
      source: "mutation",
      context: "medications",
    });
  });
});
