import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import { formatDate, formatNumber } from "@/lib/formatters";
import { METRIC_COLORS } from "@/lib/constants";
import type { Activity } from "@/features/activity/types";
import {
  aggregateBars,
  aggregationLabelSuffix,
  aggregationNote,
  periodLabel,
} from "@/lib/charts/aggregateBars";
import { ChartA11yWrapper } from "./ChartA11yWrapper";
import { rangeChartLabel } from "./a11y";
import { axisTick, tooltipProps } from "./chartTheme";
import { tooltipNumber } from "./tooltipValue";

interface ActivityChartProps {
  data: Activity[];
}

export function ActivityChart({ data }: ActivityChartProps) {
  const chartData = data
    .filter((d): d is Activity & { steps: number } => d.steps !== null)
    .map((d) => ({
      date: d.date,
      steps: d.steps,
    }));

  if (chartData.length === 0) return null;

  const { rows: shown, granularity } = aggregateBars(chartData, ["steps"]);
  const note = aggregationNote(granularity, shown.length, chartData.length, "jours");

  const ariaLabel =
    rangeChartLabel({
      title: "Histogramme du nombre de pas quotidiens",
      values: chartData.map((d) => d.steps),
      unit: "pas",
      pointNoun: "jour",
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
            tickFormatter={(v) => `${(v / 1000).toFixed(0)}k`}
            className="text-xs"
            tick={axisTick}
            width={40}
          />
          <Tooltip
            {...tooltipProps}
            labelFormatter={(d) => periodLabel(granularity, String(d))}
            formatter={(v) => {
              const n = tooltipNumber(v);
              return [n === null ? "" : formatNumber(n, 0) + " pas", "Pas"];
            }}
          />
          <Bar dataKey="steps" fill={METRIC_COLORS.steps} radius={[4, 4, 0, 0]} maxBarSize={40} />
        </BarChart>
      </ResponsiveContainer>
      {note && <p className="mt-1 text-xs text-muted-foreground">{note}</p>}
    </ChartA11yWrapper>
  );
}
