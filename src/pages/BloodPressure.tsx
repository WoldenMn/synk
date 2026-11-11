import { useState } from "react";
import { useAnySourceConnected } from "@/features/sources/hooks";
import { sourceColumn } from "@/features/sources/columns";
import { historyHint } from "@/features/sources/hint";
import { Heart, Plus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Button } from "@/components/ui/button";
import { DataTable } from "@/components/shared/DataTable";
import { PeriodEmptyState } from "@/components/shared/PeriodEmptyState";
import { ManualBPForm } from "@/features/bloodPressure/components/ManualBPForm";
import { TimeRangeSelector } from "@/components/charts/TimeRangeSelector";
import { BloodPressureChart } from "@/components/charts/BloodPressureChart";
import { useBloodPressureData, useDeleteBloodPressure } from "@/features/bloodPressure/hooks";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { useTimeRange } from "@/hooks/useTimeRange";
import { formatDateTime, formatBP } from "@/lib/formatters";
import type { BloodPressure as BPType } from "@/features/bloodPressure/types";

export default function BloodPressure() {
  const { data: withingsConnected } = useAnySourceConnected();
  const [formOpen, setFormOpen] = useState(false);
  const { timeRange, setTimeRange, from, to } = useTimeRange();
  const { data: readings, isLoading } = useBloodPressureData(from, to);
  const deleteMutation = useDeleteBloodPressure();
  const confirm = useConfirm();

  const columns = [
    {
      key: "measured_at",
      label: "Date",
      render: (row: BPType) => formatDateTime(row.measured_at),
      searchText: (row: BPType) => formatDateTime(row.measured_at),
    },
    {
      key: "bp",
      label: "Tension (mmHg)",
      render: (row: BPType) => formatBP(row.systolic, row.diastolic),
      searchText: (row: BPType) => formatBP(row.systolic, row.diastolic),
    },
    {
      key: "heart_rate",
      label: "Pouls (bpm)",
      render: (row: BPType) => (row.heart_rate != null ? String(row.heart_rate) : "\u2014"),
      searchText: (row: BPType) => (row.heart_rate != null ? String(row.heart_rate) : ""),
    },
    sourceColumn<BPType>(),
  ];

  async function handleDelete(id: number) {
    if (!(await confirm({ title: "Supprimer cette mesure de tension ?", destructive: true })))
      return;
    deleteMutation.mutate(id);
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <TimeRangeSelector value={timeRange} onChange={setTimeRange} />
        <Button onClick={() => setFormOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Ajouter
        </Button>
      </div>

      {readings && readings.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Tension artérielle</CardTitle>
          </CardHeader>
          <CardContent>
            <BloodPressureChart data={readings} />
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle>Historique</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="space-y-2 p-2">
              {[...Array(5)].map((_, i) => (
                <Skeleton key={i} className="h-10 w-full" />
              ))}
            </div>
          ) : !readings || readings.length === 0 ? (
            <PeriodEmptyState
              icon={Heart}
              periodTitle="Aucune mesure de tension sur cette période"
              noneTitle="Aucune mesure de tension enregistrée"
              noneDescription={historyHint(withingsConnected, "ou ajoutez une mesure à la main")}
              timeRange={timeRange}
              onShowAll={() => setTimeRange("all")}
            />
          ) : (
            <DataTable
              label="Mesures de tension"
              columns={columns}
              data={readings}
              onDelete={handleDelete}
              canDelete={(row) => row.source === "manual"}
              rowLabel={(row) => `la mesure de tension du ${formatDateTime(row.measured_at)}`}
              emptyMessage="Aucune donnée"
            />
          )}
        </CardContent>
      </Card>

      <ManualBPForm open={formOpen} onOpenChange={setFormOpen} />
    </div>
  );
}
