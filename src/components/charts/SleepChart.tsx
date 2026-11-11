import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { formatDate, formatDuration } from "@/lib/formatters";
import { METRIC_COLORS } from "@/lib/constants";
import type { SleepRecord } from "@/features/sleep/types";
import {
  aggregateBars,
  aggregationLabelSuffix,
  aggregationNote,
  periodLabel,
} from "@/lib/charts/aggregateBars";
import { ChartA11yWrapper } from "./ChartA11yWrapper";
import { countChartLabel } from "./a11y";
import { axisTick, legendLabel, tooltipProps } from "./chartTheme";
import { tooltipNumber } from "./tooltipValue";

const PHASE_LABELS: Record<string, string> = {
  deep: "Profond",
  light: "Léger",
  rem: "Paradoxal",
  awake: "Éveillé",
};

interface SleepChartProps {
  data: SleepRecord[];
}

export function SleepChart({ data }: SleepChartProps) {
  const chartData = data
    .filter((d) => d.duration_total)
    .map((d) => ({
      date: d.date,
      deep: (d.duration_deep || 0) / 3600,
      light: (d.duration_light || 0) / 3600,
      rem: (d.duration_rem || 0) / 3600,
      awake: (d.duration_awake || 0) / 3600,
    }));

  if (chartData.length === 0) return null;

  const { rows: shown, granularity } = aggregateBars(chartData, ["deep", "light", "rem", "awake"]);
  const note = aggregationNote(granularity, shown.length, chartData.length, "nuits");

  const ariaLabel =
    countChartLabel({
      title: "Phases de sommeil (profond, léger, paradoxal, éveillé)",
      count: chartData.length,
      pointNoun: "nuit",
    }) + aggregationLabelSuffix(granularity);

  return (
    <ChartA11yWrapper label={ariaLabel}>
      <ResponsiveContainer width="100%" height={300}>
        <BarChart
          accessibilityLayer
          data={shown}
          margin={{ top: 5, right: 20, left: 0, bottom: 5 }}
        >
          <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
          <XAxis
            dataKey="date"
            tickFormatter={(d) =>
              granularity === "month" ? periodLabel("month", d) : formatDate(d)
            }
            className="text-xs"
            tick={axisTick}
          />
          <YAxis
            tickFormatter={(v) => `${v.toFixed(0)}h`}
            className="text-xs"
            tick={axisTick}
            width={40}
          />
          <Legend
            itemSorter={null}
            formatter={(value: string) => legendLabel(PHASE_LABELS[value] || value)}
          />
          <Tooltip
            {...tooltipProps}
            labelFormatter={(d) => periodLabel(granularity, String(d))}
            formatter={(v, name) => {
              const n = tooltipNumber(v);
              const key = String(name ?? "");
              return [n === null ? "" : formatDuration(n * 3600), PHASE_LABELS[key] || key];
            }}
          />
          <Bar
            dataKey="deep"
            stackId="sleep"
            fill={METRIC_COLORS.deep_sleep}
            radius={[0, 0, 0, 0]}
          />
          <Bar dataKey="light" stackId="sleep" fill={METRIC_COLORS.light_sleep} />
          <Bar dataKey="rem" stackId="sleep" fill={METRIC_COLORS.rem_sleep} />
          <Bar dataKey="awake" stackId="sleep" fill={METRIC_COLORS.awake} radius={[4, 4, 0, 0]} />
        </BarChart>
      </ResponsiveContainer>
      {note && <p className="mt-1 text-xs text-muted-foreground">{note}</p>}
    </ChartA11yWrapper>
  );
}
