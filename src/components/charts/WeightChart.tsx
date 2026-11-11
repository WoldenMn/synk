import { useId } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
} from "recharts";
import { formatWeight } from "@/lib/formatters";
import { METRIC_COLORS } from "@/lib/constants";
import type { BodyMeasure } from "@/features/bodyMeasures/types";
import { CHART_MAX_POINTS, chartDensity, decimate } from "@/lib/charts/decimate";
import { formatTick, timeAxisFor, withTimestamp } from "@/lib/charts/timeAxis";
import { ChartA11yWrapper } from "./ChartA11yWrapper";
import { rangeChartLabel } from "./a11y";
import { axisTickSmall, tooltipProps } from "./chartTheme";
import { tooltipNumber } from "./tooltipValue";

interface WeightChartProps {
  data: BodyMeasure[];
  goalWeight?: number;
}

export function WeightChart({ data, goalWeight }: WeightChartProps) {
  const reactId = useId();
  const gradId = `weight-gradient-${reactId.replace(/:/g, "")}`;
  const chartData = withTimestamp(
    data.flatMap((d) =>
      d.weight == null ? [] : [{ date: d.measured_at.substring(0, 10), weight: d.weight }]
    )
  );

  if (chartData.length === 0) return null;

  const shown = decimate(chartData, CHART_MAX_POINTS, ["weight"]);
  const { showDots, animate } = chartDensity(shown.length);

  const weights = chartData.map((d) => d.weight);
  const allVals = goalWeight != null ? [...weights, goalWeight] : weights;
  const minW = Math.min(...allVals);
  const maxW = Math.max(...allVals);
  const pad = Math.max((maxW - minW) * 0.15, 1);

  const ariaLabel = rangeChartLabel({
    title: "Courbe du poids",
    values: weights,
    unit: "kg",
  });

  return (
    <ChartA11yWrapper label={ariaLabel}>
      <ResponsiveContainer width="100%" height={300}>
        <AreaChart
          accessibilityLayer
          data={shown}
          margin={{ top: 5, right: 20, left: 0, bottom: 5 }}
        >
          <defs>
            <linearGradient id={gradId} x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor={METRIC_COLORS.weight} stopOpacity={0.3} />
              <stop offset="95%" stopColor={METRIC_COLORS.weight} stopOpacity={0} />
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
            domain={[Math.floor(minW - pad), Math.ceil(maxW + pad)]}
            tickFormatter={(v) => `${v} kg`}
            tick={axisTickSmall}
            tickLine={false}
            axisLine={false}
            width={55}
          />
          <Tooltip
            {...tooltipProps}
            labelFormatter={(ts) => formatTick(Number(ts))}
            formatter={(v) => {
              const n = tooltipNumber(v);
              return [n === null ? "" : formatWeight(n), "Poids"];
            }}
          />
          <Area
            isAnimationActive={animate}
            type="monotone"
            dataKey="weight"
            stroke={METRIC_COLORS.weight}
            fill={`url(#${gradId})`}
            strokeWidth={2}
            dot={showDots ? { r: 3, fill: METRIC_COLORS.weight, strokeWidth: 0 } : false}
            activeDot={{ r: 5, stroke: "hsl(var(--background))", strokeWidth: 2 }}
          />
          {goalWeight != null && (
            <ReferenceLine
              y={goalWeight}
              stroke="hsl(var(--success-text))"
              strokeDasharray="5 5"
              strokeWidth={1.5}
              label={{
                value: `Objectif: ${goalWeight} kg`,
                fill: "hsl(var(--success-text))",
                fontSize: 12,
                position: "insideTopRight",
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
