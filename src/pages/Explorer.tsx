import { Compass, Database } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { DataTable } from "@/components/shared/DataTable";
import { EmptyState } from "@/components/shared/EmptyState";
import { ChartSkeleton, StatCardGridSkeleton } from "@/components/shared/LoadingSkeleton";
import { GenericMetricChart } from "@/components/charts/GenericMetricChart";
import { cn } from "@/lib/utils";
import { METRIC_CATEGORIES, METRIC_CATEGORY_LABELS } from "@/features/metrics/categoryLabels";
import { MetricSelection } from "./explorer/MetricSelection";
import { MetricStats } from "./explorer/MetricStats";
import { useExplorerState } from "./explorer/useExplorerState";

const CATEGORIES = METRIC_CATEGORIES.map((value) => ({
  value,
  label: METRIC_CATEGORY_LABELS[value],
}));

export default function Explorer() {
  const {
    category,
    setCategory,
    selectedSlug,
    setSelectedSlug,
    timeRange,
    setTimeRange,
    definitions,
    defsLoading,
    selectedDef,
    series,
    secondary,
    seriesLoading,
    stats,
    tableColumns,
    lastTen,
  } = useExplorerState();

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">Explorer les métriques</h2>
          <p className="text-sm text-muted-foreground">
            Visualisez n'importe laquelle des métriques unifiées de Synk.
          </p>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        {CATEGORIES.map((c) => (
          <Button
            key={c.value}
            variant={category === c.value ? "default" : "outline"}
            size="sm"
            aria-pressed={category === c.value}
            onClick={() => {
              setCategory(c.value);
              setSelectedSlug(null);
            }}
            className={cn("transition-all", category === c.value && "shadow-xs")}
          >
            {c.label}
          </Button>
        ))}
      </div>

      <Card>
        <CardHeader>
          <CardTitle as="h3">Sélection</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <MetricSelection
            defsLoading={defsLoading}
            definitions={definitions}
            selectedSlug={selectedSlug}
            onSelect={setSelectedSlug}
            timeRange={timeRange}
            onTimeRange={setTimeRange}
            selectedDef={selectedDef}
          />
        </CardContent>
      </Card>

      {!selectedDef ? (
        <Card>
          <CardContent className="py-12">
            <EmptyState
              icon={Compass}
              title="Choisissez une métrique"
              description="Sélectionnez une catégorie puis une métrique pour explorer son évolution dans le temps."
            />
          </CardContent>
        </Card>
      ) : (
        <>
          {seriesLoading ? (
            <StatCardGridSkeleton />
          ) : stats ? (
            <MetricStats stats={stats} selectedDef={selectedDef} />
          ) : null}

          <Card>
            <CardHeader>
              <CardTitle as="h3">{selectedDef.label}</CardTitle>
            </CardHeader>
            <CardContent>
              {seriesLoading ? (
                <ChartSkeleton />
              ) : !series || series.length === 0 ? (
                <EmptyState
                  icon={Database}
                  title="Aucune mesure pour cette métrique"
                  description="Synchronisez vos sources ou ajoutez des mesures manuelles depuis les pages dédiées."
                />
              ) : (
                <GenericMetricChart
                  data={series}
                  definition={selectedDef}
                  secondary={secondary ?? []}
                />
              )}
            </CardContent>
          </Card>

          {series && series.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle as="h3">10 dernières mesures</CardTitle>
              </CardHeader>
              <CardContent>
                <DataTable
                  label="10 dernières mesures"
                  columns={tableColumns}
                  data={lastTen.map((p, i) => ({ ...p, id: i }))}
                  emptyMessage="Aucune mesure"
                />
              </CardContent>
            </Card>
          )}
        </>
      )}
    </div>
  );
}
