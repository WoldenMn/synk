import { StatCard } from "@/components/shared/StatCard";
import { formatNumber, formatDate, orDash } from "@/lib/formatters";
import type { MeasurementType } from "@/features/manualMeasurements/types";
import type { useMeasurementStats } from "@/features/manualMeasurements/hooks";

type Stats = NonNullable<ReturnType<typeof useMeasurementStats>["data"]>;

interface Props {
  stats: Stats;
  selectedType: MeasurementType | undefined;
}

function withUnit(value: number | null, unit: string | undefined, decimals?: number): string {
  return orDash(value, (v) => `${formatNumber(v, decimals)} ${unit || ""}`);
}

function ChangeIndicator({ value, direction }: { value: number | null; direction: string | null }) {
  if (value == null) return null;
  const isGood =
    (direction === "decrease" && value < 0) ||
    (direction === "increase" && value > 0) ||
    (direction === "maintain" && Math.abs(value) < 2);
  return (
    <span
      className={`inline-flex items-center gap-0.5 text-xs font-medium ${isGood ? "text-success-text" : "text-destructive-text"}`}
    >
      {value >= 0 ? "+" : ""}
      {formatNumber(value, 1)} %
    </span>
  );
}

export function DetailStats({ stats, selectedType }: Props) {
  return (
    <div className="grid gap-4 md:grid-cols-4">
      <StatCard
        label="Dernière valeur"
        value={withUnit(stats.latest_value, selectedType?.unit)}
        sub={
          <>
            {stats.percent_change != null && (
              <ChangeIndicator
                value={stats.percent_change}
                direction={selectedType?.target_direction || null}
              />
            )}
            {stats.latest_date && (
              <p className="text-xs text-muted-foreground mt-1">{formatDate(stats.latest_date)}</p>
            )}
          </>
        }
      />
      <StatCard label="Minimum" value={withUnit(stats.min_value, selectedType?.unit)} />
      <StatCard label="Maximum" value={withUnit(stats.max_value, selectedType?.unit)} />
      <StatCard
        label="Moyenne"
        value={withUnit(stats.avg_value, selectedType?.unit, 1)}
        sub={
          <p className="text-xs text-muted-foreground">
            {stats.count} mesure{stats.count > 1 ? "s" : ""}
          </p>
        }
      />
    </div>
  );
}
