import { useId } from "react";
import {
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
  CartesianGrid,
  Area,
  AreaChart,
} from "recharts";
import { formatNumber } from "@/lib/formatters";
import type { ManualMeasurement } from "@/features/manualMeasurements/types";
import { CHART_MAX_POINTS, chartDensity, decimate } from "@/lib/charts/decimate";
import { formatTick, timeAxisFor, withTimestamp } from "@/lib/charts/timeAxis";
import { ChartA11yWrapper } from "./ChartA11yWrapper";
import { rangeChartLabel } from "./a11y";
import { axisTickSmall, tooltipProps } from "./chartTheme";
import { tooltipNumber } from "./tooltipValue";

interface Props {
  data: ManualMeasurement[];
  unit: string;
  goalValue?: number | null;
}

export function MeasurementProgressChart({ data, unit, goalValue }: Props) {
  const reactId = useId();
  const gradId = `measure-gradient-${reactId.replace(/:/g, "")}`;
  const chartData = withTimestamp(
    data.map((d) => ({
      date: d.measured_at.substring(0, 10),
      value: d.value,
    }))
  );

  if (chartData.length === 0) return null;

  const shown = decimate(chartData, CHART_MAX_POINTS, ["value"]);
  const { showDots, animate } = chartDensity(shown.length);

  const values = chartData.map((d) => d.value);
  const allValues = goalValue != null ? [...values, goalValue] : values;
  const minVal = Math.min(...allValues);
  const maxVal = Math.max(...allValues);
  const padding = (maxVal - minVal) * 0.1 || 1;

  const ariaLabel = rangeChartLabel({
    title: "Courbe de progression de la mesure",
    values,
    unit,
  });

  return (
    <ChartA11yWrapper label={ariaLabel}>
      <ResponsiveContainer width="100%" height={300}>
        <AreaChart
          accessibilityLayer
          data={shown}
          margin={{ top: 10, right: 20, bottom: 0, left: 10 }}
        >
          <defs>
            <linearGradient id={gradId} x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="hsl(var(--primary))" stopOpacity={0.2} />
              <stop offset="95%" stopColor="hsl(var(--primary))" stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" className="stroke-border" vertical={false} />
          <XAxis
            {...timeAxisFor(shown)}
            tickFormatter={formatTick}
            tick={axisTickSmall}
            tickLine={false}
            axisLine={false}
          />
          <YAxis
            domain={[minVal - padding, maxVal + padding]}
            tick={axisTickSmall}
            tickLine={false}
            axisLine={false}
            width={50}
            tickFormatter={(v) => formatNumber(v)}
          />
          <Tooltip
            formatter={(value) => {
              const n = tooltipNumber(value);
              return [n === null ? "" : `${formatNumber(n)} ${unit}`, "Valeur"];
            }}
            labelFormatter={(ts) => formatTick(Number(ts))}
            {...tooltipProps}
          />
          <Area
            isAnimationActive={animate}
            type="monotone"
            dataKey="value"
            stroke="hsl(var(--primary))"
            strokeWidth={2}
            fill={`url(#${gradId})`}
            dot={showDots ? { r: 3, fill: "hsl(var(--primary))", strokeWidth: 0 } : false}
            activeDot={{
              r: 5,
              fill: "hsl(var(--primary))",
              strokeWidth: 2,
              stroke: "hsl(var(--background))",
            }}
          />
          {goalValue != null && (
            <ReferenceLine
              y={goalValue}
              stroke="hsl(var(--warning-text))"
              strokeDasharray="6 3"
              strokeWidth={1.5}
              label={{
                value: `Objectif: ${formatNumber(goalValue)} ${unit}`,
                position: "insideTopRight",
                fontSize: 12,
                fill: "hsl(var(--warning-text))",
              }}
            />
          )}
        </AreaChart>
      </ResponsiveContainer>
      {shown.length < chartData.length && (
        <p className="mt-1 text-xs text-muted-foreground">
          {shown.length} points affichés sur {chartData.length} (moyennes par intervalle)
        </p>
      )}
    </ChartA11yWrapper>
  );
}
