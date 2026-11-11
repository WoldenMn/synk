import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell } from "recharts";
import type { Activity } from "@/features/activity/types";
import { ChartA11yWrapper } from "./ChartA11yWrapper";
import { countChartLabel } from "./a11y";
import { tooltipProps } from "./chartTheme";
import { tooltipNumber } from "./tooltipValue";

const ZONES = [
  { key: "zone0", label: "Repos", color: "hsl(var(--series-zone-rest))" },
  { key: "zone1", label: "Léger", color: "hsl(var(--series-zone-light))" },
  { key: "zone2", label: "Modéré", color: "hsl(var(--series-zone-moderate))" },
  { key: "zone3", label: "Intense", color: "hsl(var(--series-zone-intense))" },
];

interface Props {
  data: Activity[];
}

export function HrZonesChart({ data }: Props) {
  const totals = data.reduce(
    (acc, d) => {
      acc.zone0 += d.hr_zone_0 || 0;
      acc.zone1 += d.hr_zone_1 || 0;
      acc.zone2 += d.hr_zone_2 || 0;
      acc.zone3 += d.hr_zone_3 || 0;
      return acc;
    },
    { zone0: 0, zone1: 0, zone2: 0, zone3: 0 }
  );

  const total = totals.zone0 + totals.zone1 + totals.zone2 + totals.zone3;
  if (total === 0) return null;

  const chartData = ZONES.map((z) => {
    const val = totals[z.key as keyof typeof totals];
    return {
      name: z.label,
      value: val,
      percent: Math.round((val / total) * 100),
      color: z.color,
    };
  });

  const formatDur = (s: number) => {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    return h > 0 ? `${h}h${String(m).padStart(2, "0")}` : `${m} min`;
  };

  const ariaLabel = countChartLabel({
    title: "Répartition du temps par zone de fréquence cardiaque",
    count: chartData.length,
    pointNoun: "zone",
  });

  return (
    <ChartA11yWrapper label={ariaLabel}>
      <ResponsiveContainer width="100%" height={180}>
        <BarChart
          accessibilityLayer
          data={chartData}
          layout="vertical"
          margin={{ left: 70, right: 30 }}
        >
          <XAxis type="number" hide />
          <YAxis
            type="category"
            dataKey="name"
            tick={{ fontSize: 13, fill: "hsl(var(--muted-foreground))" }}
            width={60}
          />
          <Tooltip
            formatter={(value) => {
              const n = tooltipNumber(value);
              return n === null ? "" : formatDur(n);
            }}
            {...tooltipProps}
          />
          <Bar dataKey="value" radius={[0, 4, 4, 0]}>
            {chartData.map((entry, i) => (
              <Cell key={i} fill={entry.color} />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </ChartA11yWrapper>
  );
}
