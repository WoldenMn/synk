import { useState, useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { useTimeRange } from "@/hooks";
import { PageHeader } from "@/components/shared";
import { getBodyMeasures } from "@/features/bodyMeasures/api";
import { getBloodPressure } from "@/features/bloodPressure/api";
import { getSleepRecords } from "@/features/sleep/api";
import { getActivity } from "@/features/activity/api";
import { METRIC_COLORS } from "@/lib/constants";
import { getMetricOption, getDateKey, computeCorrelation } from "./Analyses/helpers";
import { MetricPicker } from "./Analyses/sections/MetricPicker";
import { CorrelationChart } from "./Analyses/sections/CorrelationChart";
import { CorrelationStats } from "./Analyses/sections/CorrelationStats";

export default function Analyses() {
  const { timeRange, setTimeRange, from, to } = useTimeRange("90d");
  const [metric1, setMetric1] = useState("weight");
  const [metric2, setMetric2] = useState("steps");

  const m1 = getMetricOption(metric1);
  const m2 = getMetricOption(metric2);

  const needsBody = m1.source === "body" || m2.source === "body";
  const needsBp = m1.source === "bp" || m2.source === "bp";
  const needsSleep = m1.source === "sleep" || m2.source === "sleep";
  const needsActivity = m1.source === "activity" || m2.source === "activity";

  const bodyQuery = useQuery({
    queryKey: ["bodyMeasures", from, to],
    queryFn: () => getBodyMeasures(from, to),
    enabled: needsBody,
  });

  const bpQuery = useQuery({
    queryKey: ["bloodPressure", from, to],
    queryFn: () => getBloodPressure(from, to),
    enabled: needsBp,
  });

  const sleepQuery = useQuery({
    queryKey: ["sleep", from, to],
    queryFn: () => getSleepRecords(from, to),
    enabled: needsSleep,
  });

  const activityQuery = useQuery({
    queryKey: ["activity", from, to],
    queryFn: () => getActivity(from, to),
    enabled: needsActivity,
  });

  const isLoading =
    (needsBody && bodyQuery.isLoading) ||
    (needsBp && bpQuery.isLoading) ||
    (needsSleep && sleepQuery.isLoading) ||
    (needsActivity && activityQuery.isLoading);

  const chartData = useMemo(() => {
    const dateMap: Record<string, { date: string; v1: number | null; v2: number | null }> = {};

    function addValue(records: Record<string, unknown>[], field: string, target: "v1" | "v2") {
      for (const rec of records) {
        const dateKey = getDateKey(rec as { measured_at?: string; date?: string });
        if (!dateKey) continue;
        if (!dateMap[dateKey]) dateMap[dateKey] = { date: dateKey, v1: null, v2: null };
        const val = rec[field];
        if (val != null && typeof val === "number") {
          dateMap[dateKey][target] = val;
        }
      }
    }

    function getRecords(source: "body" | "bp" | "sleep" | "activity"): Record<string, unknown>[] {
      switch (source) {
        case "body":
          return (bodyQuery.data ?? []) as unknown as Record<string, unknown>[];
        case "bp":
          return (bpQuery.data ?? []) as unknown as Record<string, unknown>[];
        case "sleep":
          return (sleepQuery.data ?? []) as unknown as Record<string, unknown>[];
        case "activity":
          return (activityQuery.data ?? []) as unknown as Record<string, unknown>[];
      }
    }

    addValue(getRecords(m1.source), m1.field, "v1");
    addValue(getRecords(m2.source), m2.field, "v2");

    return Object.values(dateMap).sort((a, b) => a.date.localeCompare(b.date));
  }, [
    bodyQuery.data,
    bpQuery.data,
    sleepQuery.data,
    activityQuery.data,
    m1.source,
    m1.field,
    m2.source,
    m2.field,
  ]);

  const correlation = useMemo(() => computeCorrelation(chartData), [chartData]);
  const dataPoints = chartData.filter((d) => d.v1 != null && d.v2 != null).length;

  const color1 = METRIC_COLORS[m1.value] ?? "hsl(var(--series-weight))";
  const color2 = METRIC_COLORS[m2.value] ?? "hsl(var(--series-steps))";

  return (
    <div className="space-y-6">
      <PageHeader timeRange={timeRange} onTimeRangeChange={setTimeRange} />

      <MetricPicker
        metric1={metric1}
        metric2={metric2}
        setMetric1={setMetric1}
        setMetric2={setMetric2}
      />

      <CorrelationChart
        m1={m1}
        m2={m2}
        chartData={chartData}
        isLoading={isLoading}
        color1={color1}
        color2={color2}
      />

      <CorrelationStats m1={m1} m2={m2} correlation={correlation} dataPoints={dataPoints} />
    </div>
  );
}
