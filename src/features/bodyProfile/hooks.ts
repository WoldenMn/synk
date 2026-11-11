import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { getBodySheet, addMeasurementSession, exportBodySheetPdf } from "./api";
import type { SessionEntry } from "./types";
import { reportError } from "@/lib/reportError";

export function useBodySheet() {
  return useQuery({ queryKey: ["bodySheet"], queryFn: getBodySheet });
}

export function useAddMeasurementSession() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ measuredAt, entries }: { measuredAt: string; entries: SessionEntry[] }) =>
      addMeasurementSession(measuredAt, entries),
    onSuccess: (written) => {
      qc.invalidateQueries({ queryKey: ["bodySheet"] });
      qc.invalidateQueries({ queryKey: ["measurementStats"] });
      qc.invalidateQueries({ queryKey: ["manualMeasurements"] });
      qc.invalidateQueries({ queryKey: ["clothingInputs"] });
      toast.success(
        `${written} mesure${written > 1 ? "s" : ""} enregistrée${written > 1 ? "s" : ""}`
      );
    },
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "add_measurement_session" });
    },
  });
}

export function useExportBodySheetPdf() {
  return useMutation({
    mutationFn: exportBodySheetPdf,
    onSuccess: (path) => toast.success(`Fiche exportée : ${path}`),
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "export_body_sheet_pdf" });
    },
  });
}
