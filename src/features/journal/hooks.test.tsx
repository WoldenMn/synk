import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import { useJournalEntries, useUpsertJournalEntry } from "./hooks";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

function makeWrapper(client: QueryClient) {
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return Wrapper;
}

describe("useJournalEntries", () => {
  let qc: QueryClient;

  beforeEach(() => {
    qc = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0 } },
    });
    mockInvoke.mockReset();
  });

  it("recupere les entrées via get_journal_entries avec from/to", async () => {
    const fake = [
      {
        id: 1,
        date: "2026-04-20",
        content: "Bonne journée",
        tags: null,
        created_at: "x",
        updated_at: "x",
      },
    ];
    mockInvoke.mockResolvedValue(fake);

    const { result } = renderHook(() => useJournalEntries("2026-04-01", "2026-04-30"), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockInvoke).toHaveBeenCalledWith("get_journal_entries", {
      from: "2026-04-01",
      to: "2026-04-30",
    });
    expect(result.current.data).toEqual(fake);
  });

  it("useUpsertJournalEntry invalide ['journal'] apres succès", async () => {
    mockInvoke.mockResolvedValue(42);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useUpsertJournalEntry(), {
      wrapper: makeWrapper(qc),
    });

    await act(async () => {
      await result.current.mutateAsync({
        date: "2026-04-29",
        content: "Test",
      });
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["journal"] });
  });
});
