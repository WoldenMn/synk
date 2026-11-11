import { Ruler, Plus, ChevronRight, Settings2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/shared/EmptyState";
import { LoadingSkeleton } from "@/components/shared/LoadingSkeleton";
import { TimeRangeSelector } from "@/components/charts/TimeRangeSelector";
import { useMemo } from "react";
import { useMeasurementOverview } from "@/features/manualMeasurements/hooks";
import { useTimeRange } from "@/hooks/useTimeRange";
import { formatNumber } from "@/lib/formatters";
import type { MeasurementOverview, MeasurementType } from "@/features/manualMeasurements/types";

interface MeasurementGridProps {
  types: MeasurementType[] | undefined;
  isLoading: boolean;
  onSelect: (typeId: number) => void;
  onAddEntry: () => void;
  onManage: () => void;
}

export function MeasurementGrid({
  types,
  isLoading,
  onSelect,
  onAddEntry,
  onManage,
}: MeasurementGridProps) {
  const { timeRange, setTimeRange, from, to } = useTimeRange();
  const { data: overview = [] } = useMeasurementOverview(from, to);
  const byType = useMemo(() => new Map(overview.map((o) => [o.type_id, o])), [overview]);

  const builtinTypes = types?.filter((t) => t.is_builtin === 1) || [];
  const customTypes = types?.filter((t) => t.is_builtin === 0) || [];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">Suivez vos mesures corporelles</p>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={onManage}>
            <Settings2 className="mr-2 h-4 w-4" />
            Gérer
          </Button>
          <Button size="sm" onClick={onAddEntry}>
            <Plus className="mr-2 h-4 w-4" />
            Mesure
          </Button>
        </div>
      </div>

      <TimeRangeSelector value={timeRange} onChange={setTimeRange} />

      {isLoading ? (
        <LoadingSkeleton rows={6} height="h-24" />
      ) : !types || types.length === 0 ? (
        <EmptyState
          icon={Ruler}
          title="Aucun type de mensuration"
          description="Créez un type de mensuration pour commencer le suivi"
        />
      ) : (
        <>
          {builtinTypes.length > 0 && (
            <div>
              <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                Tours de corps
              </h2>
              <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
                {builtinTypes.map((t) => (
                  <MeasurementTypeCard
                    key={t.id}
                    type={t}
                    overview={byType.get(t.id)}
                    onClick={() => onSelect(t.id)}
                  />
                ))}
              </div>
            </div>
          )}

          {customTypes.length > 0 && (
            <div>
              <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                Personnalises
              </h2>
              <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
                {customTypes.map((t) => (
                  <MeasurementTypeCard
                    key={t.id}
                    type={t}
                    overview={byType.get(t.id)}
                    onClick={() => onSelect(t.id)}
                  />
                ))}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
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

function GoalProgress({
  current,
  target,
  direction,
  unit,
}: {
  current: number | null;
  target: number;
  direction: string | null;
  unit: string;
}) {
  if (current == null) return null;
  let pct: number;
  if (direction === "decrease") {
    const start = target * 1.5;
    pct = Math.max(0, Math.min(100, ((start - current) / (start - target)) * 100));
  } else if (direction === "increase") {
    pct = Math.max(0, Math.min(100, (current / target) * 100));
  } else {
    const diff = Math.abs(current - target);
    pct = Math.max(0, 100 - (diff / target) * 100);
  }
  const color = pct >= 90 ? "bg-success" : pct >= 50 ? "bg-warning" : "bg-destructive";
  return (
    <div className="mt-2">
      <div className="flex items-center justify-between text-xs text-muted-foreground mb-1">
        <span>
          Objectif : {target} {unit}
        </span>
        <span>{Math.round(pct)}%</span>
      </div>
      <div className="h-1.5 rounded-full bg-muted overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-500 ${color}`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}

function MiniChart({ data }: { data: number[] }) {
  if (data.length < 2) return null;
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;
  const h = 32;
  const w = 80;
  const points = data
    .map((v, i) => `${(i / (data.length - 1)) * w},${h - ((v - min) / range) * h}`)
    .join(" ");
  return (
    <svg width={w} height={h} className="shrink-0 opacity-60">
      <polyline
        points={points}
        fill="none"
        stroke="hsl(var(--primary))"
        strokeWidth={1.5}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

type OverviewStats = NonNullable<MeasurementOverview["stats"]>;

function CardMeta({
  stats,
  direction,
}: {
  stats: OverviewStats | undefined;
  direction: MeasurementType["target_direction"];
}) {
  return (
    <div className="flex items-center gap-2 flex-wrap">
      {stats?.percent_change != null && (
        <ChangeIndicator value={stats.percent_change} direction={direction} />
      )}
      {stats?.count != null && stats.count > 0 && (
        <span className="text-xs text-muted-foreground">
          {stats.count} mesure{stats.count > 1 ? "s" : ""}
        </span>
      )}
      {stats?.min_value != null && stats?.max_value != null && stats.count > 1 && (
        <span className="text-xs text-muted-foreground">
          {formatNumber(stats.min_value)} – {formatNumber(stats.max_value)}
        </span>
      )}
    </div>
  );
}

function MeasurementTypeCard({
  type: t,
  overview,
  onClick,
}: {
  type: MeasurementType;
  overview: MeasurementOverview | undefined;
  onClick: () => void;
}) {
  const stats = overview?.stats;
  const sparkData = overview?.values ?? [];

  return (
    <button
      onClick={onClick}
      className="flex items-center justify-between rounded-lg border bg-card p-4 text-left transition-all hover:bg-accent/15 hover:shadow-md hover:border-primary/20 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 group"
    >
      <div className="space-y-1.5 min-w-0 flex-1">
        <p className="text-sm font-medium truncate">{t.label}</p>
        <div className="flex items-center gap-2">
          <span className="text-xl font-bold">
            {stats?.latest_value != null ? formatNumber(stats.latest_value) : "\u2014"}
          </span>
          <span className="text-xs text-muted-foreground">{t.unit}</span>
        </div>
        <CardMeta stats={stats} direction={t.target_direction} />
        {t.target_value != null && (
          <GoalProgress
            current={stats?.latest_value ?? null}
            target={t.target_value}
            direction={t.target_direction}
            unit={t.unit}
          />
        )}
      </div>
      <div className="flex flex-col items-end gap-2 ml-3">
        <MiniChart data={sparkData} />
        <ChevronRight className="h-5 w-5 text-muted-foreground shrink-0 opacity-0 group-hover:opacity-100 transition-opacity" />
      </div>
    </button>
  );
}
