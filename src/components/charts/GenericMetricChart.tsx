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
import { METRIC_COLORS } from "@/lib/constants";
import type { MetricDefinition, MetricPoint } from "@/features/metrics/types";
import { CHART_MAX_POINTS, chartDensity, decimate } from "@/lib/charts/decimate";
import { formatTick, timeAxisFor, withTimestamp } from "@/lib/charts/timeAxis";
import { mergeSecondary } from "@/lib/charts/mergeSecondary";
import { sourceLabel } from "@/features/sources/labels";
import { ChartA11yWrapper } from "./ChartA11yWrapper";
import { rangeChartLabel } from "./a11y";
import { axisTickSmall, tooltipProps } from "./chartTheme";
import { tooltipNumber } from "./tooltipValue";

interface GenericMetricChartProps {
  data: MetricPoint[];
  definition: MetricDefinition;
  secondary?: MetricPoint[];
  target?: number;
  height?: number;
}

const FALLBACK_COLOR = "hsl(var(--primary))";

function getColor(slug: string): string {
  return METRIC_COLORS[slug] ?? FALLBACK_COLOR;
}

function formatValue(value: number, decimals: number, unit: string): string {
  if (!Number.isFinite(value)) return "—";
  const v = value.toFixed(decimals);
  return unit ? `${v} ${unit}` : v;
}

export function GenericMetricChart({
  data,
  definition,
  secondary = [],
  target,
  height = 320,
}: GenericMetricChartProps) {
  // useId React garantit un ID unique par instance, evitant les collisions SVG
  // si plusieurs charts du meme slug sont montes simultanement.
  const reactId = useId();

  if (data.length === 0) return null;

  const chartData = withTimestamp(
    mergeSecondary(
      data
        .map((p) => ({
          date: p.measured_at.substring(0, 10),
          value: p.value,
          source: p.source,
        }))
        .filter((d) => Number.isFinite(d.value)),
      secondary
    )
  );

  if (chartData.length === 0) return null;

  const shown = decimate(chartData, CHART_MAX_POINTS, ["value", "other"]);
  const { showDots, animate } = chartDensity(shown.length);

  const values = chartData.map((d) => d.value);
  const otherValues = chartData.flatMap((d) => (d.other === undefined ? [] : [d.other]));
  const otherSources = [
    ...new Set(chartData.flatMap((d) => (d.otherSource === undefined ? [] : [d.otherSource]))),
  ].map(sourceLabel);
  const plotted = [...values, ...otherValues];
  const allVals = target != null && Number.isFinite(target) ? [...plotted, target] : plotted;
  const minV = Math.min(...allVals);
  const maxV = Math.max(...allVals);
  const range = maxV - minV;
  const pad = range > 0 ? range * 0.15 : Math.max(Math.abs(maxV) * 0.1, 1);

  const color = getColor(definition.slug);
  const gradId = `metric-gradient-${reactId.replace(/:/g, "")}`;

  const ariaLabel = rangeChartLabel({
    title: `Courbe : ${definition.label}`,
    values,
    unit: definition.unit,
  });
  const fullLabel =
    otherValues.length > 0
      ? `${ariaLabel} ${otherValues.length} point${otherValues.length > 1 ? "s" : ""} d'une autre source en retrait.`
      : ariaLabel;

  return (
    <ChartA11yWrapper label={fullLabel}>
      <ResponsiveContainer width="100%" height={height}>
        <AreaChart
          accessibilityLayer
          data={shown}
          margin={{ top: 5, right: 20, left: 0, bottom: 5 }}
        >
          <defs>
            <linearGradient id={gradId} x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor={color} stopOpacity={0.3} />
              <stop offset="95%" stopColor={color} stopOpacity={0} />
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
            domain={[Math.floor(minV - pad), Math.ceil(maxV + pad)]}
            tickFormatter={(v) =>
              `${Number(v).toFixed(definition.decimals)}${definition.unit ? ` ${definition.unit}` : ""}`
            }
            tick={axisTickSmall}
            tickLine={false}
            axisLine={false}
            width={70}
          />
          <Tooltip
            {...tooltipProps}
            labelFormatter={(ts) => formatTick(Number(ts))}
            formatter={(v, _name, item) => {
              const n = tooltipNumber(v);
              const payload = item.payload as { source?: string; otherSource?: string } | undefined;
              const source = item.dataKey === "other" ? payload?.otherSource : payload?.source;
              return [
                n === null ? "" : formatValue(n, definition.decimals, definition.unit),
                source ? sourceLabel(source) : definition.label,
              ];
            }}
          />
          <Area
            isAnimationActive={animate}
            type="monotone"
            dataKey="value"
            stroke={color}
            fill={`url(#${gradId})`}
            strokeWidth={2}
            dot={showDots ? { r: 3, fill: color, strokeWidth: 0 } : false}
            activeDot={{ r: 5, stroke: "hsl(var(--background))", strokeWidth: 2 }}
          />
          {otherValues.length > 0 && (
            <Area
              isAnimationActive={false}
              type="monotone"
              dataKey="other"
              stroke="none"
              fill="none"
              legendType="none"
              dot={{ r: 3.5, fill: "hsl(var(--muted-foreground))", strokeWidth: 0 }}
              activeDot={{ r: 5, stroke: "hsl(var(--background))", strokeWidth: 2 }}
            />
          )}
          {target != null && (
            <ReferenceLine
              y={target}
              stroke="hsl(var(--success-text))"
              strokeDasharray="5 5"
              strokeWidth={1.5}
              label={{
                value: `Objectif : ${formatValue(target, definition.decimals, definition.unit)}`,
                fill: "hsl(var(--success-text))",
                fontSize: 12,
                position: "insideTopRight",
              }}
            />
          )}
        </AreaChart>
      </ResponsiveContainer>
      {otherSources.length > 0 && (
        <p className="mt-1 text-xs text-muted-foreground">
          Points en retrait : {otherSources.join(", ")} (source non retenue pour la courbe)
        </p>
      )}
      {shown.length < chartData.length && (
        <p className="mt-1 text-xs text-muted-foreground">
          {shown.length} points affichés sur {chartData.length} (moyennes par intervalle)
        </p>
      )}
    </ChartA11yWrapper>
  );
}
