import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import {
  getClothingInputs,
  getClothingSizes,
  setClothingSize,
  deleteClothingSize,
  saveEstimatedSizes,
} from "./api";
import { reportError } from "@/lib/reportError";

export function useClothingInputs() {
  return useQuery({ queryKey: ["clothingInputs"], queryFn: getClothingInputs });
}

export function useClothingSizes() {
  return useQuery({ queryKey: ["clothingSizes"], queryFn: getClothingSizes });
}

export function useSetClothingSize() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      category,
      value,
      source,
    }: {
      category: string;
      value: string;
      source: "override" | "manual";
    }) => setClothingSize(category, value, source),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["clothingSizes"] });
      toast.success("Taille enregistrée");
    },
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "set_clothing_size" });
    },
  });
}

export function useDeleteClothingSize() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (category: string) => deleteClothingSize(category),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["clothingSizes"] });
      toast.success("Taille réinitialisée");
    },
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "delete_clothing_size" });
    },
  });
}

export function useSaveEstimatedSizes() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (sizes: { category: string; value: string }[]) => saveEstimatedSizes(sizes),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["clothingSizes"] });
      qc.invalidateQueries({ queryKey: ["profileCard"] });
    },
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "save_estimated_sizes", toast: false });
    },
  });
}
