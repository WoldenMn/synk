import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";

export interface CrudHookConfig<T, AddInput> {
  domain: string;
  list: (from: string, to: string) => Promise<T[]>;
  latest: () => Promise<T | null>;
  add: (data: AddInput) => Promise<number>;
  remove: (id: number) => Promise<void>;
  staleTime?: number;
  toasts?: {
    addSuccess?: string;
    deleteSuccess?: string;
  };
}

export function createCrudHook<T, AddInput>(config: CrudHookConfig<T, AddInput>) {
  const { domain, list, latest, add, remove, staleTime, toasts } = config;
  const addSuccess = toasts?.addSuccess ?? "Mesure ajoutée";
  const deleteSuccess = toasts?.deleteSuccess ?? "Mesure supprimée";

  return {
    useList: (from: string, to: string, enabled = true) =>
      useQuery({
        queryKey: [domain, from, to],
        queryFn: () => list(from, to),
        enabled: enabled && !!from && !!to,
        ...(staleTime !== undefined && { staleTime }),
        placeholderData: keepPreviousData,
      }),

    useLatest: () =>
      useQuery({
        queryKey: [domain, "latest"],
        queryFn: latest,
        ...(staleTime !== undefined && { staleTime }),
      }),

    useAdd: () => {
      const qc = useQueryClient();
      return useMutation({
        mutationFn: (data: AddInput) => add(data),
        onSuccess: () => {
          qc.invalidateQueries({ queryKey: [domain] });
          toast.success(addSuccess);
        },
        onError: (e: unknown) => {
          reportError(e, { source: "mutation", context: domain });
        },
      });
    },

    useDelete: () => {
      const qc = useQueryClient();
      return useMutation({
        mutationFn: (id: number) => remove(id),
        onSuccess: () => {
          qc.invalidateQueries({ queryKey: [domain] });
          toast.success(deleteSuccess);
        },
        onError: (e: unknown) => {
          reportError(e, { source: "mutation", context: domain });
        },
      });
    },
  };
}
