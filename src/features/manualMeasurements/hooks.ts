import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  getMeasurementTypes,
  addMeasurementType,
  deleteMeasurementType,
  getManualMeasurements,
  addManualMeasurement,
  deleteManualMeasurement,
  getMeasurementStats,
  getMeasurementOverview,
  setMeasurementGoal,
} from "./api";
import type { AddMeasurementTypeInput, AddManualMeasurementInput } from "./types";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";

export function useMeasurementTypes() {
  return useQuery({
    queryKey: ["measurementTypes"],
    queryFn: getMeasurementTypes,
    staleTime: 5 * 60 * 1000,
  });
}

export function useAddMeasurementType() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: AddMeasurementTypeInput) => addMeasurementType(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["measurementTypes"] });
      toast.success("Type de mensuration ajouté");
    },
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "add_measurement_type" });
    },
  });
}

export function useDeleteMeasurementType() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => deleteMeasurementType(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["measurementTypes"] });
      toast.success("Type supprimé");
    },
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "delete_measurement_type" });
    },
  });
}

export function useManualMeasurements(typeId: number, from: string, to: string) {
  return useQuery({
    queryKey: ["manualMeasurements", typeId, from, to],
    queryFn: () => getManualMeasurements(typeId, from, to),
    enabled: !!typeId && !!from && !!to,
  });
}

export function useAddManualMeasurement() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: AddManualMeasurementInput) => addManualMeasurement(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["manualMeasurements"] });
      queryClient.invalidateQueries({ queryKey: ["measurementStats"] });
      queryClient.invalidateQueries({ queryKey: ["clothingInputs"] });
    },
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "add_manual_measurement" });
    },
  });
}

export function useDeleteManualMeasurement() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => deleteManualMeasurement(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["manualMeasurements"] });
      queryClient.invalidateQueries({ queryKey: ["measurementStats"] });
      queryClient.invalidateQueries({ queryKey: ["clothingInputs"] });
      toast.success("Mensuration supprimée");
    },
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "delete_manual_measurement" });
    },
  });
}

export function useMeasurementStats(typeId: number, from: string, to: string) {
  return useQuery({
    queryKey: ["measurementStats", typeId, from, to],
    queryFn: () => getMeasurementStats(typeId, from, to),
    enabled: typeId > 0,
    staleTime: 5 * 60 * 1000,
  });
}

export function useMeasurementOverview(from: string, to: string) {
  return useQuery({
    queryKey: ["measurementStats", "overview", from, to],
    queryFn: () => getMeasurementOverview(from, to),
    enabled: !!from && !!to,
    staleTime: 5 * 60 * 1000,
  });
}

export function useSetMeasurementGoal() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      typeId,
      targetValue,
      targetDirection,
    }: {
      typeId: number;
      targetValue: number | null;
      targetDirection: string | null;
    }) => setMeasurementGoal(typeId, targetValue, targetDirection),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["measurementTypes"] });
      qc.invalidateQueries({ queryKey: ["measurementStats"] });
      toast.success("Objectif mis à jour");
    },
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "set_measurement_goal" });
    },
  });
}
