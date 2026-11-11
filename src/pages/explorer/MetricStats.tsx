import { StatCard } from "@/components/shared/StatCard";
import { formatNumber } from "@/lib/formatters";
import { cn } from "@/lib/utils";
import type { MetricDefinition } from "@/features/metrics/types";
import type { ExplorerStats } from "./useExplorerState";

export function MetricStats({
  stats,
  selectedDef,
}: {
  stats: ExplorerStats;
  selectedDef: MetricDefinition;
}) {
  return (
    <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
      <StatCard
        label="Dernière valeur"
        value={`${formatNumber(stats.latest, selectedDef.decimals)}${selectedDef.unit ? ` ${selectedDef.unit}` : ""}`}
        glow="indigo"
      />
      <StatCard
        label="Moyenne"
        value={`${formatNumber(stats.avg, selectedDef.decimals)}${selectedDef.unit ? ` ${selectedDef.unit}` : ""}`}
        glow="emerald"
      />
      <StatCard
        label="Min - Max"
        value={`${formatNumber(stats.min, selectedDef.decimals)} - ${formatNumber(stats.max, selectedDef.decimals)}`}
        sub={
          selectedDef.unit ? (
            <span className="text-xs text-muted-foreground">{selectedDef.unit}</span>
          ) : undefined
        }
        glow="violet"
      />
      <StatCard
        label="Mesures"
        value={String(stats.count)}
        sub={
          <span
            className={cn(
              "text-xs",
              stats.delta > 0
                ? "text-success-text"
                : stats.delta < 0
                  ? "text-destructive-text"
                  : "text-muted-foreground"
            )}
          >
            {stats.delta > 0 ? "+" : ""}
            {formatNumber(stats.delta, selectedDef.decimals)} sur la période
          </span>
        }
        glow="amber"
      />
    </div>
  );
}
