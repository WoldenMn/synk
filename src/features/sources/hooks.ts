import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { getMetricPriorities, listSources, setMetricPriority, sourceSetCredentials } from "./api";
import { DASHBOARD_CARDS_KEY } from "@/features/dashboard/hooks";
import { reportError } from "@/lib/reportError";
import { logError } from "@/lib/logger";
import type { SourceEvent } from "./events";

export const SOURCES_KEY = ["sources"] as const;

export interface CredentialsInput {
  clientId: string;
  clientSecret: string;
}

export function useSources() {
  return useQuery({
    queryKey: SOURCES_KEY,
    queryFn: listSources,
    staleTime: 60 * 1000,
    throwOnError: false,
    meta: { surface: "shell" },
  });
}

export function useAnySourceConnected() {
  return useQuery({
    queryKey: SOURCES_KEY,
    queryFn: listSources,
    staleTime: 60 * 1000,
    throwOnError: false,
    meta: { surface: "shell" },
    select: (sources) => sources.some((s) => s.connected),
  });
}

export function useSourceCredentials(source: string) {
  return useQuery({
    queryKey: SOURCES_KEY,
    queryFn: listSources,
    staleTime: 60 * 1000,
    throwOnError: false,
    meta: { surface: "shell" },
    select: (sources) => sources.find((s) => s.id === source)?.credentials_configured ?? false,
  });
}

export function useSetCredentials(source: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CredentialsInput) =>
      sourceSetCredentials(source, input.clientId, input.clientSecret),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: SOURCES_KEY });
    },
    onError: (e: unknown) => {
      reportError(e, { source: "mutation", context: "sourceCredentials" });
    },
  });
}

export function useSourceEvents(
  event: SourceEvent,
  sourceIds: readonly string[],
  handler: (source: string, payload: unknown) => void
) {
  const key = sourceIds.join(",");
  useEffect(() => {
    let cancelled = false;
    const off: (() => void)[] = [];
    for (const id of key === "" ? [] : key.split(",")) {
      listen<unknown>(`${id}-${event}`, (e) => handler(id, e.payload))
        .then((fn) => {
          if (cancelled) fn();
          else off.push(fn);
        })
        .catch((e) => logError(`écoute de ${id}-${event} impossible:`, e));
    }
    return () => {
      cancelled = true;
      off.forEach((fn) => fn());
    };
  }, [event, key, handler]);
}

export const PRIORITIES_KEY = ["metric-priorities"] as const;

export function useMetricPriorities() {
  return useQuery({
    queryKey: PRIORITIES_KEY,
    queryFn: getMetricPriorities,
    staleTime: 60 * 1000,
  });
}

export function useSetMetricPriority() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: { slug: string; sources: string[] }) =>
      setMetricPriority(input.slug, input.sources),
    onSuccess: () => {
      for (const queryKey of [
        PRIORITIES_KEY,
        ["metric-series"],
        ["metric-latest"],
        ["insights"],
        [DASHBOARD_CARDS_KEY],
      ]) {
        qc.invalidateQueries({ queryKey: [...queryKey] });
      }
    },
    onError: (e: unknown) => {
      reportError(e, { source: "mutation", context: "metricPriority" });
    },
  });
}
