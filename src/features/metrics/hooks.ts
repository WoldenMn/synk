// Stack Metriques V2 unifiee.
// Lit la table `metrics` + tables `*_legacy` via metric_repository::read_metric_series (dedup automatique).
// N'altere PAS les hooks legacy (useBodyMeasures, useManualMeasurements...) qui restent en place.

import { useQuery } from "@tanstack/react-query";
import {
  getMetricDefinitions,
  getMetricSeries,
  getMetricLatest,
  getMetricSecondaryPoints,
} from "@/features/metrics/api";
import type { MetricCategory } from "@/features/metrics/types";

export function useMetricDefinitions(category?: MetricCategory) {
  return useQuery({
    queryKey: ["metric-definitions", category ?? "all"],
    queryFn: () => getMetricDefinitions(category),
    staleTime: 30 * 60 * 1000,
  });
}

export function useMetricSeries(slug: string, from: string, to: string, enabled = true) {
  return useQuery({
    queryKey: ["metric-series", slug, from, to],
    queryFn: () => getMetricSeries(slug, from, to),
    enabled: enabled && !!slug && !!from && !!to,
    staleTime: 5 * 60 * 1000,
  });
}

export function useMetricSecondary(slug: string, from: string, to: string, enabled = true) {
  return useQuery({
    queryKey: ["metric-series", "secondary", slug, from, to],
    queryFn: () => getMetricSecondaryPoints(slug, from, to),
    enabled: enabled && !!slug && !!from && !!to,
    staleTime: 5 * 60 * 1000,
  });
}

export function useMetricLatest(slug: string, enabled = true) {
  return useQuery({
    queryKey: ["metric-latest", slug],
    queryFn: () => getMetricLatest(slug),
    enabled: enabled && !!slug,
    staleTime: 5 * 60 * 1000,
  });
}
