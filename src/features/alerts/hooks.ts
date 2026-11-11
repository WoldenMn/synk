import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";
import {
  getAlerts,
  addAlert,
  deleteAlert,
  getAlertEvents,
  markAlertsSeen,
  checkAlerts,
} from "./api";
import type { AddAlertInput } from "./types";

export function useAlerts() {
  return useQuery({ queryKey: ["alerts"], queryFn: getAlerts, staleTime: 30_000 });
}

export function useAlertEvents() {
  return useQuery({ queryKey: ["alerts", "events"], queryFn: getAlertEvents, staleTime: 30_000 });
}

export function useAddAlert() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (d: AddAlertInput) => addAlert(d),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["alerts"] });
      toast.success("Alerte ajoutée");
    },
    onError: (e: unknown) => reportError(e, { source: "mutation", context: "alerts" }),
  });
}

export function useDeleteAlert() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => deleteAlert(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["alerts"] });
      toast.success("Alerte supprimée");
    },
    onError: (e: unknown) => reportError(e, { source: "mutation", context: "alerts" }),
  });
}

export function useCheckAlerts() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => checkAlerts(),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["alerts", "events"] }),
    onError: (e: unknown) => reportError(e, { source: "mutation", context: "alerts" }),
  });
}

export function useMarkAlertsSeen() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => markAlertsSeen(),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["alerts", "events"] }),
    onError: (e: unknown) => reportError(e, { source: "mutation", context: "alerts" }),
  });
}
