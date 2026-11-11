import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getAuditMetrics, getAuditPurgeConfig, purgeAuditLog, setAuditPurgeConfig } from "./api";
import type { AuditPurgeConfig } from "./types";

export function useAuditMetrics() {
  return useQuery({
    queryKey: ["audit_metrics"],
    queryFn: getAuditMetrics,
    staleTime: 30_000,
  });
}

export function usePurgeAuditLog() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (olderThanDays: number) => purgeAuditLog(olderThanDays),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["audit_metrics"] });
      qc.invalidateQueries({ queryKey: ["db_metrics"] });
    },
  });
}

export function useAuditPurgeConfig() {
  return useQuery({
    queryKey: ["audit_purge_config"],
    queryFn: getAuditPurgeConfig,
    staleTime: 60_000,
  });
}

export function useSetAuditPurgeConfig() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (config: AuditPurgeConfig) => setAuditPurgeConfig(config),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["audit_purge_config"] });
    },
  });
}
