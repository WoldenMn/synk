import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { getSyncLog, clearSyncLog, getLastSuccessfulSync } from "./api";

export function useSyncLog(limit?: number) {
  return useQuery({
    queryKey: ["sync_log", limit],
    queryFn: () => getSyncLog(limit),
    staleTime: 30_000,
  });
}

export function useClearSyncLog() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: clearSyncLog,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["sync_log"] }),
  });
}

export function useLastSuccessfulSync() {
  return useQuery({
    queryKey: ["sync", "last-success"],
    queryFn: getLastSuccessfulSync,
    staleTime: 0,
    throwOnError: false,
    meta: { surface: "shell" },
  });
}
