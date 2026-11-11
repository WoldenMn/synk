import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  getJournalEntries,
  upsertJournalEntry,
  deleteJournalEntry,
  type JournalInput,
} from "./api";

export function useJournalEntries(from?: string, to?: string) {
  return useQuery({
    queryKey: ["journal", from, to],
    queryFn: () => getJournalEntries(from, to),
    staleTime: 30_000,
  });
}

export function useUpsertJournalEntry() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: JournalInput) => upsertJournalEntry(data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["journal"] }),
  });
}

export function useDeleteJournalEntry() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => deleteJournalEntry(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["journal"] }),
  });
}
