import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { METRIC_COLORS } from "@/lib/constants";
import type { BodyMeasure } from "@/features/bodyMeasures/types";
import { CHART_MAX_POINTS, chartDensity, decimate } from "@/lib/charts/decimate";
import { formatTick, timeAxisFor, withTimestamp } from "@/lib/charts/timeAxis";
import { ChartA11yWrapper } from "./ChartA11yWrapper";
import { countChartLabel } from "./a11y";
import { axisTickSmall, tooltipProps } from "./chartTheme";
import { tooltipNumber } from "./tooltipValue";

interface BodyCompositionChartProps {
  data: BodyMeasure[];
}

type SeriesKey = "muscle_mass" | "fat_mass" | "bone_mass";

const SERIES: { key: SeriesKey; label: string; color: string }[] = [
  { key: "muscle_mass", label: "Muscle", color: METRIC_COLORS.muscle_mass ?? "" },
  { key: "fat_mass", label: "Graisse", color: METRIC_COLORS.fat_ratio ?? "" },
  { key: "bone_mass", label: "Os", color: METRIC_COLORS.bone_mass ?? "" },
];

type Row = { date: string; ts: number } & Record<SeriesKey, number | null>;

function SeriesChart({
  rows,
  series,
}: {
  rows: Row[];
  series: { key: SeriesKey; label: string; color: string };
}) {
  const points = rows.filter((r) => r[series.key] != null);
  const shown = decimate(points, CHART_MAX_POINTS, [series.key]);
  const { showDots, animate } = chartDensity(shown.length);
  const label = countChartLabel({
    title: `Courbe : masse ${series.label.toLowerCase()}`,
    count: points.length,
    pointNoun: "mesure",
  });

  return (
    <div>
      <p className="mb-2 text-sm font-medium">{series.label}</p>
      <ChartA11yWrapper label={label}>
        <ResponsiveContainer width="100%" height={240}>
          <LineChart
            accessibilityLayer
            data={shown}
            margin={{ top: 5, right: 16, left: 0, bottom: 5 }}
          >
            <CartesianGrid strokeDasharray="3 3" className="stroke-border" vertical={false} />
            <XAxis
              {...timeAxisFor(shown)}
              tickFormatter={formatTick}
              tick={axisTickSmall}
              tickLine={false}
              axisLine={false}
              minTickGap={32}
            />
            <YAxis
              domain={["auto", "auto"]}
              tickFormatter={(v) => `${Number(v).toFixed(1)} kg`}
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
                return [n !== null ? `${n.toFixed(1)} kg` : "—", series.label];
              }}
            />
            <Line
              isAnimationActive={animate}
              type="monotone"
              dataKey={series.key}
              stroke={series.color}
              strokeWidth={2}
              dot={showDots ? { r: 2 } : false}
              connectNulls
            />
          </LineChart>
        </ResponsiveContainer>
      </ChartA11yWrapper>
    </div>
  );
}

export function BodyCompositionChart({ data }: BodyCompositionChartProps) {
  const rows: Row[] = withTimestamp(
    data
      .filter((d) => d.fat_mass != null || d.muscle_mass != null)
      .map((d) => ({
        date: d.measured_at.substring(0, 10),
        fat_mass: d.fat_mass,
        muscle_mass: d.muscle_mass,
        bone_mass: d.bone_mass,
      }))
  );

  if (rows.length === 0) return null;

  const present = SERIES.filter((s) => rows.some((r) => r[s.key] != null));

  return (
    <div className="grid gap-4 lg:grid-cols-3">
      {present.map((s) => (
        <SeriesChart key={s.key} rows={rows} series={s} />
      ))}
    </div>
  );
}
