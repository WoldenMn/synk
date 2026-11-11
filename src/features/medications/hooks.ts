import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";
import {
  getMedications,
  addMedication,
  updateMedication,
  deleteMedication,
  logMedicationTaken,
  getMedicationLogs,
} from "./api";
import type { AddMedicationInput, UpdateMedicationInput, LogMedicationInput } from "./types";

export function useMedications() {
  return useQuery({ queryKey: ["medications"], queryFn: getMedications, staleTime: 30_000 });
}

export function useMedicationLogs(medicationId: number, from: string, to: string) {
  return useQuery({
    queryKey: ["medications", "logs", medicationId, from, to],
    queryFn: () => getMedicationLogs(medicationId, from, to),
    enabled: medicationId > 0 && !!from && !!to,
  });
}

function mutationOpts(qc: ReturnType<typeof useQueryClient>, successMsg: string) {
  return {
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["medications"] });
      toast.success(successMsg);
    },
    onError: (e: unknown) =>
      reportError(e, { source: "mutation" as const, context: "medications" }),
  };
}

export function useAddMedication() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (d: AddMedicationInput) => addMedication(d),
    ...mutationOpts(qc, "Médicament ajouté"),
  });
}

export function useUpdateMedication() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (d: UpdateMedicationInput) => updateMedication(d),
    ...mutationOpts(qc, "Médicament mis à jour"),
  });
}

export function useDeleteMedication() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => deleteMedication(id),
    ...mutationOpts(qc, "Médicament supprimé"),
  });
}

export function useLogMedication() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (d: LogMedicationInput) => logMedicationTaken(d),
    ...mutationOpts(qc, "Prise enregistrée"),
  });
}
