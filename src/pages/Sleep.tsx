import { useState } from "react";
import { useAnySourceConnected } from "@/features/sources/hooks";
import { historyHint } from "@/features/sources/hint";
import { PeriodEmptyState } from "@/components/shared/PeriodEmptyState";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Moon } from "lucide-react";
import { StatCard, LoadingSkeleton, PageHeader, DataTable } from "@/components/shared";
import { SleepChart } from "@/components/charts/SleepChart";
import { ManualSleepForm } from "@/features/sleep/components/ManualSleepForm";
import { useSleepData, useDeleteSleep } from "@/features/sleep/hooks";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { useTimeRange } from "@/hooks/useTimeRange";
import { formatDate, formatDuration, formatNumber } from "@/lib/formatters";
import { avg } from "@/lib/stats";
import { buildDetailedMetrics, sleepColumns } from "./sleep/sleepMetrics";

function apneaBadge(value: number | null) {
  if (value == null) return null;
  if (value < 5) return <Badge className="bg-success/10 text-success-text">Normal</Badge>;
  if (value < 15) return <Badge className="bg-warning/10 text-warning-text">Léger</Badge>;
  if (value < 30) return <Badge className="bg-warning/10 text-warning-text">Modéré</Badge>;
  return <Badge className="bg-destructive/10 text-destructive-text">Sévère</Badge>;
}

export default function Sleep() {
  const { data: withingsConnected } = useAnySourceConnected();
  const [formOpen, setFormOpen] = useState(false);
  const { timeRange, setTimeRange, from, to } = useTimeRange();
  const { data: sleepRecords, isLoading } = useSleepData(from, to);
  const deleteSleep = useDeleteSleep();
  const confirm = useConfirm();

  const records = sleepRecords || [];

  async function handleDelete(id: number) {
    if (!(await confirm({ title: "Supprimer cet enregistrement de sommeil ?", destructive: true })))
      return;
    deleteSleep.mutate(id);
  }

  const avgDuration = avg(records, "duration_total");
  const avgScore = avg(records, "sleep_score");
  const avgEfficiency = avg(records, "sleep_efficiency");
  const avgApnea = avg(records, "apnea_hypopnea_index");
  const detailedMetrics = buildDetailedMetrics(records);

  return (
    <div className="space-y-6">
      <PageHeader
        timeRange={timeRange}
        onTimeRangeChange={setTimeRange}
        onAdd={() => setFormOpen(true)}
      />

      <div className="grid gap-4 md:grid-cols-4">
        <StatCard
          label="Durée moyenne"
          value={avgDuration != null ? formatDuration(avgDuration) : "\u2014"}
        />
        <StatCard
          label="Score moyen"
          value={avgScore != null ? `${Math.round(avgScore)}/100` : "\u2014"}
        />
        <StatCard
          label="Efficacité (%)"
          value={avgEfficiency != null ? `${formatNumber(avgEfficiency, 1)} %` : "\u2014"}
        />
        <StatCard
          label="Indice d'apnée"
          value={avgApnea != null ? formatNumber(avgApnea, 1) : "\u2014"}
          sub={apneaBadge(avgApnea)}
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Phases de sommeil</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <LoadingSkeleton rows={6} />
          ) : records.length > 0 ? (
            <SleepChart data={records} />
          ) : (
            <PeriodEmptyState
              icon={Moon}
              periodTitle="Aucune nuit sur cette période"
              noneTitle="Aucune nuit enregistrée"
              noneDescription={historyHint(withingsConnected, "ou ajoutez une nuit à la main")}
              timeRange={timeRange}
              onShowAll={() => setTimeRange("all")}
            />
          )}
        </CardContent>
      </Card>

      {records.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Données détaillées</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
              {detailedMetrics.map((m) => (
                <div key={m.label} className="space-y-1">
                  <p className="text-xs text-muted-foreground">{m.label}</p>
                  <p className="text-sm font-medium">{m.value}</p>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {records.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Historique</CardTitle>
          </CardHeader>
          <CardContent>
            <DataTable
              label="Nuits de sommeil"
              columns={sleepColumns}
              data={records}
              onDelete={handleDelete}
              canDelete={(row) => row.source === "manual"}
              rowLabel={(row) => `la nuit du ${formatDate(row.date)}`}
              emptyMessage="Aucune donnée de sommeil"
            />
          </CardContent>
        </Card>
      )}

      <ManualSleepForm open={formOpen} onOpenChange={setFormOpen} />
    </div>
  );
}
