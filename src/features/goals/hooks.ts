import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { getGoals, upsertGoal, deleteGoal } from "./api";
import type { UpsertGoalInput } from "./types";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";

export function useGoals() {
  return useQuery({
    queryKey: ["goals"],
    queryFn: getGoals,
    staleTime: 5 * 60 * 1000,
  });
}

export function useUpsertGoal() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: UpsertGoalInput) => upsertGoal(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["goals"] });
      toast.success("Objectif enregistré");
    },
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "upsert_goal" });
    },
  });
}

export function useDeleteGoal() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (metricKey: string) => deleteGoal(metricKey),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["goals"] });
      toast.success("Objectif supprimé");
    },
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "delete_goal" });
    },
  });
}
