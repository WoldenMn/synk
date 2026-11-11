import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";
import { getLabResults, addLabResult, deleteLabResult } from "./api";
import type { AddLabResultInput } from "./types";

export function useLabResults(from: string, to: string) {
  return useQuery({
    queryKey: ["labs", from, to],
    queryFn: () => getLabResults(from, to),
    enabled: !!from && !!to,
    staleTime: 30_000,
  });
}

export function useAddLabResult() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: AddLabResultInput) => addLabResult(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["labs"] });
      toast.success("Résultat ajouté");
    },
    onError: (e: unknown) => reportError(e, { source: "mutation", context: "labs" }),
  });
}

export function useDeleteLabResult() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => deleteLabResult(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["labs"] });
      toast.success("Résultat supprimé");
    },
    onError: (e: unknown) => reportError(e, { source: "mutation", context: "labs" }),
  });
}
