import { useState } from "react";
import { useAnySourceConnected } from "@/features/sources/hooks";
import { historyHint } from "@/features/sources/hint";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Dumbbell, Plus } from "lucide-react";
import { TimeRangeSelector } from "@/components/charts/TimeRangeSelector";
import { PeriodEmptyState } from "@/components/shared/PeriodEmptyState";
import { DataTable } from "@/components/shared/DataTable";
import type { Workout } from "@/features/workouts/types";
import { sourceColumn } from "@/features/sources/columns";
import { useWorkoutsData, useDeleteWorkout } from "@/features/workouts/hooks";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { useTimeRange } from "@/hooks/useTimeRange";
import { formatNumber, formatDate } from "@/lib/formatters";
import { ManualWorkoutForm } from "@/features/workouts/components/ManualWorkoutForm";

function formatWorkoutDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = String(Math.floor((seconds % 3600) / 60)).padStart(2, "0");
  return `${h}h${m}`;
}

export default function Workouts() {
  const { data: withingsConnected } = useAnySourceConnected();
  const { timeRange, setTimeRange, from, to } = useTimeRange();
  const { data: workouts, isLoading } = useWorkoutsData(from, to);
  const deleteMutation = useDeleteWorkout();
  const confirm = useConfirm();
  const [formOpen, setFormOpen] = useState(false);

  const totalWorkouts = workouts?.length ?? 0;

  const totalDuration = workouts?.length
    ? workouts.reduce((sum, w) => sum + (w.duration || 0), 0)
    : null;

  const totalDistance = workouts?.length
    ? workouts.reduce((sum, w) => sum + (w.distance || 0), 0)
    : null;

  const totalCalories = workouts?.length
    ? workouts.reduce((sum, w) => sum + (w.calories || 0), 0)
    : null;

  async function handleDelete(id: number) {
    if (!(await confirm({ title: "Supprimer cet entraînement ?", destructive: true }))) return;
    deleteMutation.mutate(id);
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <Button variant="outline" size="sm" onClick={() => setFormOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Ajouter
        </Button>
        <TimeRangeSelector value={timeRange} onChange={setTimeRange} />
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Entraînements</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{totalWorkouts}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Durée totale</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {totalDuration ? `${formatNumber(totalDuration / 3600)} h` : "\u2014"}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Distance totale</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {totalDistance ? `${formatNumber(totalDistance / 1000)} km` : "\u2014"}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Calories totales</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {totalCalories ? `${formatNumber(totalCalories, 0)} kcal` : "\u2014"}
            </p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Historique des entraînements</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="space-y-2 p-2">
              {[...Array(6)].map((_, i) => (
                <Skeleton key={i} className="h-10 w-full" />
              ))}
            </div>
          ) : workouts && workouts.length > 0 ? (
            <DataTable
              label="Entraînements"
              columns={[
                {
                  key: "date",
                  label: "Date",
                  render: (w) => formatDate(w.date),
                  searchText: (w) => formatDate(w.date),
                },
                {
                  key: "category_label",
                  label: "Type",
                  searchText: (w) => w.category_label || "Autre",
                  render: (w) => <Badge variant="secondary">{w.category_label || "Autre"}</Badge>,
                },
                {
                  key: "duration",
                  label: "Durée",
                  render: (w) => (w.duration ? formatWorkoutDuration(w.duration) : "\u2014"),
                  searchText: (w) => (w.duration ? formatWorkoutDuration(w.duration) : ""),
                },
                {
                  key: "distance",
                  label: "Distance",
                  render: (w) => (w.distance ? `${formatNumber(w.distance / 1000)} km` : "\u2014"),
                  searchText: (w) => (w.distance ? `${formatNumber(w.distance / 1000)} km` : ""),
                },
                {
                  key: "calories",
                  label: "Calories",
                  render: (w) => (w.calories ? `${formatNumber(w.calories, 0)} kcal` : "\u2014"),
                  searchText: (w) => (w.calories ? `${formatNumber(w.calories, 0)} kcal` : ""),
                },
                {
                  key: "hr_average",
                  label: "FC moy",
                  render: (w) => (w.hr_average ? `${w.hr_average} bpm` : "\u2014"),
                  searchText: (w) => (w.hr_average ? `${w.hr_average} bpm` : ""),
                },
                sourceColumn<Workout>(),
              ]}
              data={workouts}
              onDelete={handleDelete}
              rowLabel={(w) => `l'entraînement du ${formatDate(w.date)}`}
              canDelete={(w) => w.source === "manual"}
              searchable={false}
              emptyMessage="Aucun entraînement"
            />
          ) : (
            <PeriodEmptyState
              icon={Dumbbell}
              periodTitle="Aucun entraînement sur cette période"
              noneTitle="Aucun entraînement enregistré"
              noneDescription={historyHint(
                withingsConnected,
                "ou ajoutez un entraînement à la main"
              )}
              timeRange={timeRange}
              onShowAll={() => setTimeRange("all")}
            />
          )}
        </CardContent>
      </Card>

      <ManualWorkoutForm open={formOpen} onOpenChange={setFormOpen} />
    </div>
  );
}
