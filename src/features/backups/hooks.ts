import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { createBackup } from "@/features/export/api";
import { reportError } from "@/lib/reportError";
import { listBackups } from "./api";

export function useBackups() {
  return useQuery({ queryKey: ["backups"], queryFn: listBackups });
}

export function useCreateBackup() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: createBackup,
    onSuccess: (path: string) => {
      qc.invalidateQueries({ queryKey: ["backups"] });
      toast.success("Sauvegarde créée", { description: path });
    },
    onError: (e: unknown) => {
      reportError(e, { source: "mutation", context: "createBackup" });
    },
  });
}
