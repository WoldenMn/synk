import { invoke } from "@tauri-apps/api/core";
import type { MetricDefinition, MetricPoint } from "@/features/metrics/types";

export function getMetricDefinitions(category?: string): Promise<MetricDefinition[]> {
  return invoke<MetricDefinition[]>("get_metric_definitions", { category });
}

export function getMetricSeries(slug: string, from: string, to: string): Promise<MetricPoint[]> {
  return invoke<MetricPoint[]>("get_metric_series", { slug, from, to });
}

export function getMetricLatest(slug: string): Promise<MetricPoint | null> {
  return invoke<MetricPoint | null>("get_metric_latest", { slug });
}

export function getMetricSecondaryPoints(
  slug: string,
  from: string,
  to: string
): Promise<MetricPoint[]> {
  return invoke<MetricPoint[]>("get_metric_secondary_points", { slug, from, to });
}
