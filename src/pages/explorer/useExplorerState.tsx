import { useMemo, useState } from "react";
import {
  useMetricDefinitions,
  useMetricSecondary,
  useMetricSeries,
} from "@/features/metrics/hooks";
import { sourceColumn } from "@/features/sources/columns";
import { useTimeRange } from "@/hooks/useTimeRange";
import { formatDateTime, formatNumber } from "@/lib/formatters";
import type { MetricCategory, MetricDefinition, MetricPoint } from "@/features/metrics/types";

export function useExplorerState() {
  const [category, setCategory] = useState<MetricCategory>("body_composition");
  const [selectedSlug, setSelectedSlug] = useState<string | null>(null);
  const { timeRange, setTimeRange, from, to } = useTimeRange();

  const { data: definitions, isLoading: defsLoading } = useMetricDefinitions(category);
  const selectedDef = useMemo<MetricDefinition | undefined>(
    () => definitions?.find((d) => d.slug === selectedSlug),
    [definitions, selectedSlug]
  );

  const { data: series, isLoading: seriesLoading } = useMetricSeries(
    selectedSlug ?? "",
    from,
    to,
    !!selectedSlug
  );

  const { data: secondary } = useMetricSecondary(selectedSlug ?? "", from, to, !!selectedSlug);

  const stats = useMemo(() => {
    if (!series || series.length === 0) return null;
    const values = series.map((p) => p.value);
    const latest = series[series.length - 1]?.value ?? 0;
    const first = series[0]?.value ?? latest;
    return {
      latest,
      min: Math.min(...values),
      max: Math.max(...values),
      avg: values.reduce((a, b) => a + b, 0) / values.length,
      count: series.length,
      delta: latest - first,
    };
  }, [series]);

  const tableColumns = useMemo(
    () => [
      {
        key: "measured_at",
        label: "Date",
        render: (row: MetricPoint) => formatDateTime(row.measured_at),
        searchText: (row: MetricPoint) => formatDateTime(row.measured_at),
      },
      {
        key: "value",
        label: "Valeur",
        render: (row: MetricPoint) =>
          `${formatNumber(row.value, selectedDef?.decimals ?? 1)}${selectedDef?.unit ? ` ${selectedDef.unit}` : ""}`,
        searchText: (row: MetricPoint) =>
          `${formatNumber(row.value, selectedDef?.decimals ?? 1)}${selectedDef?.unit ? ` ${selectedDef.unit}` : ""}`,
      },
      sourceColumn<MetricPoint>(),
    ],
    [selectedDef]
  );

  const lastTen = useMemo(() => {
    if (!series) return [];
    return [...series].reverse().slice(0, 10);
  }, [series]);

  return {
    category,
    setCategory,
    selectedSlug,
    setSelectedSlug,
    timeRange,
    setTimeRange,
    definitions,
    defsLoading,
    selectedDef,
    series,
    secondary,
    seriesLoading,
    stats,
    tableColumns,
    lastTen,
  };
}

export type ExplorerStats = NonNullable<ReturnType<typeof useExplorerState>["stats"]>;
