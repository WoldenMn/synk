import { useState } from "react";
import { useAnySourceConnected } from "@/features/sources/hooks";
import { sourceColumn } from "@/features/sources/columns";
import { historyHint } from "@/features/sources/hint";
import { PeriodEmptyState } from "@/components/shared/PeriodEmptyState";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Footprints } from "lucide-react";
import { StatCard, LoadingSkeleton, PageHeader, DataTable } from "@/components/shared";
import { ActivityChart } from "@/components/charts/ActivityChart";
import { HrZonesChart } from "@/components/charts/HrZonesChart";
import { ManualActivityForm } from "@/features/activity/components/ManualActivityForm";
import { useActivityData, useDeleteActivity } from "@/features/activity/hooks";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { useTimeRange } from "@/hooks/useTimeRange";
import { formatNumber, formatDuration, formatDate } from "@/lib/formatters";
import { avg, sum } from "@/lib/stats";
import type { Activity as ActivityRecord } from "@/features/activity/types";

export default function Activity() {
  const { data: withingsConnected } = useAnySourceConnected();
  const [formOpen, setFormOpen] = useState(false);
  const { timeRange, setTimeRange, from, to } = useTimeRange();
  const { data: activityData, isLoading } = useActivityData(from, to);
  const deleteActivity = useDeleteActivity();
  const confirm = useConfirm();

  const records = activityData || [];
  const avgSteps = avg(records, "steps");
  const totalDistance = records.length ? sum(records, "distance") : null;
  const totalCalories = records.length ? sum(records, "calories_active") : null;
  const avgActive = avg(records, "duration_active");

  async function handleDelete(id: number) {
    if (!(await confirm({ title: "Supprimer cette journée d'activité ?", destructive: true })))
      return;
    deleteActivity.mutate(id);
  }

  const activityColumns = [
    {
      key: "date",
      label: "Date",
      render: (row: ActivityRecord) => formatDate(row.date),
      searchText: (row: ActivityRecord) => formatDate(row.date),
    },
    {
      key: "steps",
      label: "Pas",
      render: (row: ActivityRecord) => (row.steps != null ? formatNumber(row.steps, 0) : "\u2014"),
      searchText: (row: ActivityRecord) => (row.steps != null ? formatNumber(row.steps, 0) : ""),
    },
    {
      key: "calories_active",
      label: "Calories",
      render: (row: ActivityRecord) =>
        row.calories_active != null ? `${formatNumber(row.calories_active, 0)} kcal` : "\u2014",
      searchText: (row: ActivityRecord) =>
        row.calories_active != null ? `${formatNumber(row.calories_active, 0)} kcal` : "",
    },
    {
      key: "duration_active",
      label: "Durée active",
      render: (row: ActivityRecord) =>
        row.duration_active != null ? formatDuration(row.duration_active) : "\u2014",
      searchText: (row: ActivityRecord) =>
        row.duration_active != null ? formatDuration(row.duration_active) : "",
    },
    sourceColumn<ActivityRecord>(),
  ];

  return (
    <div className="space-y-6">
      <PageHeader
        timeRange={timeRange}
        onTimeRangeChange={setTimeRange}
        onAdd={() => setFormOpen(true)}
      />

      <div className="grid gap-4 md:grid-cols-4">
        <StatCard
          label="Pas/jour (moy.)"
          value={avgSteps ? formatNumber(Math.round(avgSteps), 0) : "\u2014"}
        />
        <StatCard
          label="Distance totale"
          value={totalDistance ? `${formatNumber(totalDistance / 1000)} km` : "\u2014"}
        />
        <StatCard
          label="Calories actives"
          value={totalCalories ? `${formatNumber(totalCalories, 0)} kcal` : "\u2014"}
        />
        <StatCard
          label="Temps actif/jour"
          value={avgActive ? formatDuration(avgActive) : "\u2014"}
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Pas quotidiens</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <LoadingSkeleton rows={6} />
          ) : records.length > 0 ? (
            <ActivityChart data={records} />
          ) : (
            <PeriodEmptyState
              icon={Footprints}
              periodTitle="Aucune activité sur cette période"
              noneTitle="Aucune activité enregistrée"
              noneDescription={historyHint(
                withingsConnected,
                "pour importer vos pas et vos calories"
              )}
              timeRange={timeRange}
              onShowAll={() => setTimeRange("all")}
            />
          )}
        </CardContent>
      </Card>
      {records.some(
        (d) =>
          d.hr_zone_0 != null || d.hr_zone_1 != null || d.hr_zone_2 != null || d.hr_zone_3 != null
      ) && (
        <Card>
          <CardHeader>
            <CardTitle>Zones de fréquence cardiaque</CardTitle>
          </CardHeader>
          <CardContent>
            <HrZonesChart data={records} />
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
              label="Journées d'activité"
              columns={activityColumns}
              data={records}
              onDelete={handleDelete}
              canDelete={(row) => row.source === "manual"}
              rowLabel={(row) => `la journée du ${formatDate(row.date)}`}
              emptyMessage="Aucune donnée d'activité"
            />
          </CardContent>
        </Card>
      )}

      <ManualActivityForm open={formOpen} onOpenChange={setFormOpen} />
    </div>
  );
}
