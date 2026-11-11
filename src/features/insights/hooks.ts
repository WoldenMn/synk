import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { getInsights, generateInsights, markInsightsSeen } from "./api";

export function useInsights(limit?: number) {
  return useQuery({
    queryKey: ["insights", limit],
    queryFn: () => getInsights(limit),
    staleTime: 60_000,
  });
}

export function useGenerateInsights() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: generateInsights,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["insights"] }),
  });
}

export function useMarkInsightsSeen() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: markInsightsSeen,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["insights"] }),
  });
}
