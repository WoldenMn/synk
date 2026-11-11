import {
  ComposedChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  CartesianGrid,
  Legend,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { CHART_MAX_POINTS, chartDensity, decimate } from "@/lib/charts/decimate";
import { formatNumber } from "@/lib/formatters";
import { formatTick, timeAxisFor, withTimestamp } from "@/lib/charts/timeAxis";
import { axisTick, legendLabel, tooltipProps } from "@/components/charts/chartTheme";
import { ChartA11yWrapper } from "@/components/charts/ChartA11yWrapper";
import { countChartLabel } from "@/components/charts/a11y";
import { tooltipNumber } from "@/components/charts/tooltipValue";
import type { MetricOption } from "../helpers";

type ChartPoint = {
  date: string;
  v1: number | null;
  v2: number | null;
};

interface Props {
  m1: MetricOption;
  m2: MetricOption;
  chartData: ChartPoint[];
  isLoading: boolean;
  color1: string;
  color2: string;
}

export function CorrelationChart({ m1, m2, chartData, isLoading, color1, color2 }: Props) {
  const shown = decimate(withTimestamp(chartData), CHART_MAX_POINTS, ["v1", "v2"]);
  const { animate } = chartDensity(shown.length);
  return (
    <Card>
      <CardHeader>
        <CardTitle>
          {m1.label} vs {m2.label}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex h-80 items-center justify-center text-muted-foreground">
            Chargement...
          </div>
        ) : chartData.length === 0 ? (
          <div className="flex h-80 items-center justify-center text-muted-foreground">
            Aucune donnée pour cette période
          </div>
        ) : (
          <ChartA11yWrapper
            label={countChartLabel({
              title: `Courbes superposées : ${m1.label} et ${m2.label}`,
              count: chartData.length,
              pointNoun: "jour",
            })}
          >
            <ResponsiveContainer width="100%" height={380}>
              <ComposedChart
                accessibilityLayer
                data={shown}
                margin={{ top: 5, right: 10, left: 10, bottom: 5 }}
              >
                <CartesianGrid strokeDasharray="3 3" className="stroke-muted/30" />
                <XAxis {...timeAxisFor(shown)} tickFormatter={formatTick} tick={axisTick} />
                <YAxis
                  yAxisId="left"
                  tick={axisTick}
                  label={{
                    value: `${m1.label} (${m1.unit})`,
                    angle: -90,
                    position: "insideLeft",
                    style: { fontSize: 12, fill: "hsl(var(--muted-foreground))" },
                  }}
                />
                <YAxis
                  yAxisId="right"
                  orientation="right"
                  tick={axisTick}
                  label={{
                    value: `${m2.label} (${m2.unit})`,
                    angle: 90,
                    position: "insideRight",
                    style: { fontSize: 12, fill: "hsl(var(--muted-foreground))" },
                  }}
                />
                <Legend
                  itemSorter={null}
                  formatter={(value: string) => legendLabel(value === "v1" ? m1.label : m2.label)}
                />
                <Tooltip
                  {...tooltipProps}
                  labelFormatter={(ts) => formatTick(Number(ts))}
                  formatter={(value, name) => {
                    const n = tooltipNumber(value);
                    const text = n === null ? "" : formatNumber(n);
                    return [text, name === "v1" ? m1.label : m2.label];
                  }}
                />
                <Line
                  yAxisId="left"
                  isAnimationActive={animate}
                  type="monotone"
                  dataKey="v1"
                  stroke={color1}
                  strokeWidth={2}
                  dot={false}
                  connectNulls
                  name="v1"
                />
                <Line
                  yAxisId="right"
                  isAnimationActive={animate}
                  type="monotone"
                  dataKey="v2"
                  stroke={color2}
                  strokeWidth={2}
                  dot={false}
                  connectNulls
                  name="v2"
                />
              </ComposedChart>
            </ResponsiveContainer>
          </ChartA11yWrapper>
        )}
        {shown.length < chartData.length && (
          <p className="mt-1 text-xs text-muted-foreground">
            {shown.length} points affichés sur {chartData.length} (moyennes par intervalle)
          </p>
        )}
      </CardContent>
    </Card>
  );
}
