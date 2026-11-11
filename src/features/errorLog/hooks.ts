import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";
import { getErrorLog, purgeErrorLog } from "./api";

export function useErrorLog(codeFilter: string | null = null) {
  return useQuery({
    queryKey: ["errorLog", codeFilter],
    queryFn: () => getErrorLog(200, codeFilter),
  });
}

export function usePurgeErrorLog() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: purgeErrorLog,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["errorLog"] });
      toast.success("Journal d'erreurs vide");
    },
    onError: (e: unknown) => {
      reportError(e, { source: "mutation", context: "purge_error_log" });
    },
  });
}
