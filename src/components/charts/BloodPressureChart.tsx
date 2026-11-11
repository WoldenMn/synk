import {
  ComposedChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceArea,
} from "recharts";
import { METRIC_COLORS, BP_ZONES } from "@/lib/constants";
import type { BloodPressure } from "@/features/bloodPressure/types";
import { CHART_MAX_POINTS, chartDensity, decimate } from "@/lib/charts/decimate";
import { formatTick, timeAxisFor, withTimestamp } from "@/lib/charts/timeAxis";
import { ChartA11yWrapper } from "./ChartA11yWrapper";
import { countChartLabel } from "./a11y";
import { axisTick, tooltipProps } from "./chartTheme";
import { tooltipNumber } from "./tooltipValue";

interface BloodPressureChartProps {
  data: BloodPressure[];
}

export function BloodPressureChart({ data }: BloodPressureChartProps) {
  const chartData = withTimestamp(
    data.map((d) => ({
      date: d.measured_at.substring(0, 10),
      systolic: d.systolic,
      diastolic: d.diastolic,
      heart_rate: d.heart_rate,
    }))
  );

  if (chartData.length === 0) return null;

  const shown = decimate(chartData, CHART_MAX_POINTS, ["systolic", "diastolic", "heart_rate"]);
  const { showDots, animate } = chartDensity(shown.length);

  const ariaLabel = countChartLabel({
    title: "Tension artérielle systolique et diastolique",
    count: chartData.length,
    pointNoun: "mesure",
  });

  return (
    <ChartA11yWrapper label={ariaLabel}>
      <ResponsiveContainer width="100%" height={300}>
        <ComposedChart
          accessibilityLayer
          data={shown}
          margin={{ top: 5, right: 20, left: 0, bottom: 5 }}
        >
          <CartesianGrid strokeDasharray="3 3" className="stroke-border" />

          {BP_ZONES.slice(0, 4).map((zone) => (
            <ReferenceArea
              key={zone.label}
              y1={zone.systolicMin}
              y2={zone.systolicMax}
              fill={zone.color}
              fillOpacity={0.06}
            />
          ))}

          <XAxis
            {...timeAxisFor(shown)}
            tickFormatter={formatTick}
            className="text-xs"
            tick={axisTick}
          />
          <YAxis
            domain={[40, 200]}
            tickFormatter={(v) => `${v}`}
            className="text-xs"
            tick={axisTick}
            width={40}
          />
          <Tooltip
            {...tooltipProps}
            labelFormatter={(ts) => formatTick(Number(ts))}
            formatter={(v, name) => {
              const n = tooltipNumber(v);
              const key = String(name ?? "");
              const labels: Record<string, string> = {
                systolic: "Systolique",
                diastolic: "Diastolique",
                heart_rate: "FC",
              };
              const units: Record<string, string> = {
                systolic: "mmHg",
                diastolic: "mmHg",
                heart_rate: "bpm",
              };
              return [n === null ? "" : `${n} ${units[key] || ""}`, labels[key] || key];
            }}
          />
          <Line
            isAnimationActive={animate}
            type="monotone"
            dataKey="systolic"
            stroke={METRIC_COLORS.systolic}
            strokeWidth={2}
            dot={showDots ? { r: 3 } : false}
          />
          <Line
            isAnimationActive={animate}
            type="monotone"
            dataKey="diastolic"
            stroke={METRIC_COLORS.diastolic}
            strokeWidth={2}
            dot={showDots ? { r: 3 } : false}
          />
          <Line
            isAnimationActive={animate}
            type="monotone"
            dataKey="heart_rate"
            stroke={METRIC_COLORS.heart_rate}
            strokeWidth={1.5}
            strokeDasharray="4 4"
            dot={false}
          />
        </ComposedChart>
      </ResponsiveContainer>
      {shown.length < chartData.length && (
        <p className="mt-1 text-xs text-muted-foreground">
          {shown.length} points affichés sur {chartData.length} (moyennes par intervalle)
        </p>
      )}
    </ChartA11yWrapper>
  );
}
