import { Skeleton } from "@/components/ui/skeleton";
import { MetricSelector } from "@/components/shared/MetricSelector";
import { TimeRangeSelector } from "@/components/charts/TimeRangeSelector";
import { useTimeRange } from "@/hooks/useTimeRange";
import type { MetricDefinition } from "@/features/metrics/types";

interface Props {
  defsLoading: boolean;
  definitions: MetricDefinition[] | undefined;
  selectedSlug: string | null;
  onSelect: (slug: string | null) => void;
  timeRange: ReturnType<typeof useTimeRange>["timeRange"];
  onTimeRange: ReturnType<typeof useTimeRange>["setTimeRange"];
  selectedDef: MetricDefinition | undefined;
}

export function MetricSelection({
  defsLoading,
  definitions,
  selectedSlug,
  onSelect,
  timeRange,
  onTimeRange,
  selectedDef,
}: Props) {
  if (defsLoading) return <Skeleton className="h-9 w-full" />;
  if (!definitions || definitions.length === 0) {
    return <p className="text-sm text-muted-foreground">Aucune métrique dans cette catégorie.</p>;
  }
  return (
    <>
      <MetricSelector
        definitions={definitions}
        value={selectedSlug}
        onChange={onSelect}
        placeholder={`Choisir parmi ${definitions.length} métriques...`}
      />
      <div className="flex items-center justify-between gap-4">
        <TimeRangeSelector value={timeRange} onChange={onTimeRange} />
        {selectedDef && (
          <span className="text-xs text-muted-foreground">
            {selectedDef.unit ? `Unité : ${selectedDef.unit}` : "Sans unité"}
          </span>
        )}
      </div>
    </>
  );
}
