import { useState } from "react";
import { useAnySourceConnected } from "@/features/sources/hooks";
import { sourceColumn } from "@/features/sources/columns";
import { historyHint } from "@/features/sources/hint";
import { PeriodEmptyState } from "@/components/shared/PeriodEmptyState";
import { Scale } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { StatCard, LoadingSkeleton, PageHeader, DataTable } from "@/components/shared";
import { ManualWeightForm } from "@/features/bodyMeasures/components/ManualWeightForm";
import { WeightChart } from "@/components/charts/WeightChart";
import { BodyCompositionChart } from "@/components/charts/BodyCompositionChart";
import { useBodyMeasures, useDeleteBodyMeasure } from "@/features/bodyMeasures/hooks";
import { useGoals } from "@/features/goals/hooks";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { useTimeRange } from "@/hooks/useTimeRange";
import {
  formatDateTime,
  formatWeight,
  formatPercent,
  formatNumber,
  orDash,
} from "@/lib/formatters";

import type { BodyMeasure } from "@/features/bodyMeasures/types";

const formatKg = (v: number) => `${formatNumber(v)} kg`;

function LatestStats({ latest }: { latest: BodyMeasure | null }) {
  return (
    <div className="grid gap-4 md:grid-cols-4">
      <StatCard
        label="Poids"
        value={orDash(latest?.weight, formatWeight)}
        glow="indigo"
        delay={0}
      />
      <StatCard
        label="Masse grasse"
        value={orDash(latest?.fat_ratio, formatPercent)}
        sub={
          latest?.fat_mass != null ? (
            <span className="text-xs text-muted-foreground">
              {formatNumber(latest.fat_mass)} kg
            </span>
          ) : null
        }
        glow="amber"
        delay={60}
      />
      <StatCard
        label="Masse musculaire"
        value={orDash(latest?.muscle_mass, formatKg)}
        glow="emerald"
        delay={120}
      />
      <StatCard
        label="Hydratation"
        value={orDash(latest?.hydration, formatWeight)}
        sub={
          latest?.bone_mass != null ? (
            <span className="text-xs text-muted-foreground">
              Os : {formatNumber(latest.bone_mass)} kg
            </span>
          ) : null
        }
        glow="violet"
        delay={180}
      />
    </div>
  );
}

export default function BodyComposition() {
  const { data: withingsConnected } = useAnySourceConnected();
  const [formOpen, setFormOpen] = useState(false);
  const { timeRange, setTimeRange, from, to } = useTimeRange();
  const { data: measures, isLoading } = useBodyMeasures(from, to);
  const { data: goals } = useGoals();
  const deleteMutation = useDeleteBodyMeasure();
  const confirm = useConfirm();
  const goalWeight = goals?.find((g) => g.metric_key === "weight")?.target_value;

  const latest = measures?.[measures.length - 1] ?? null;

  const columns = [
    {
      key: "measured_at",
      label: "Date",
      render: (row: BodyMeasure) => formatDateTime(row.measured_at),
      searchText: (row: BodyMeasure) => formatDateTime(row.measured_at),
    },
    {
      key: "weight",
      label: "Poids",
      render: (row: BodyMeasure) => orDash(row.weight, formatWeight),
      searchText: (row: BodyMeasure) => (row.weight != null ? formatWeight(row.weight) : ""),
    },
    {
      key: "fat_ratio",
      label: "Graisse %",
      render: (row: BodyMeasure) => orDash(row.fat_ratio, formatPercent),
      searchText: (row: BodyMeasure) => (row.fat_ratio != null ? formatPercent(row.fat_ratio) : ""),
    },
    {
      key: "muscle_mass",
      label: "Muscle",
      render: (row: BodyMeasure) => orDash(row.muscle_mass, formatKg),
      searchText: (row: BodyMeasure) => (row.muscle_mass != null ? formatKg(row.muscle_mass) : ""),
    },
    {
      key: "bone_mass",
      label: "Os",
      render: (row: BodyMeasure) => orDash(row.bone_mass, formatKg),
      searchText: (row: BodyMeasure) => (row.bone_mass != null ? formatKg(row.bone_mass) : ""),
    },
    {
      key: "hydration",
      label: "Eau",
      render: (row: BodyMeasure) => orDash(row.hydration, formatWeight),
      searchText: (row: BodyMeasure) => (row.hydration != null ? formatWeight(row.hydration) : ""),
    },
    sourceColumn<BodyMeasure>(),
  ];

  async function handleDelete(id: number) {
    if (!(await confirm({ title: "Supprimer cette pesée ?", destructive: true }))) return;
    deleteMutation.mutate(id);
  }

  return (
    <div className="space-y-6">
      <PageHeader
        timeRange={timeRange}
        onTimeRangeChange={setTimeRange}
        onAdd={() => setFormOpen(true)}
        addLabel="Pesée"
      />

      <LatestStats latest={latest} />

      {measures && measures.length > 0 && (
        <div className="grid gap-4 md:grid-cols-2">
          <Card className="animate-fade-in-up" style={{ animationDelay: "240ms" }}>
            <CardHeader>
              <CardTitle>Évolution du poids</CardTitle>
            </CardHeader>
            <CardContent>
              <WeightChart data={measures} goalWeight={goalWeight} />
            </CardContent>
          </Card>
          <Card className="animate-fade-in-up" style={{ animationDelay: "300ms" }}>
            <CardHeader>
              <CardTitle>Composition corporelle</CardTitle>
            </CardHeader>
            <CardContent>
              <BodyCompositionChart data={measures} />
            </CardContent>
          </Card>
        </div>
      )}

      <Card className="animate-fade-in-up" style={{ animationDelay: "360ms" }}>
        <CardHeader>
          <CardTitle>Historique</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <LoadingSkeleton />
          ) : !measures || measures.length === 0 ? (
            <PeriodEmptyState
              icon={Scale}
              periodTitle="Aucune pesée sur cette période"
              noneTitle="Aucune pesée enregistrée"
              noneDescription={historyHint(withingsConnected, "ou ajoutez une pesée à la main")}
              timeRange={timeRange}
              onShowAll={() => setTimeRange("all")}
            />
          ) : (
            <DataTable
              label="Pesées"
              columns={columns}
              data={measures}
              onDelete={handleDelete}
              canDelete={(row) => row.source === "manual"}
              rowLabel={(row) => `la pesée du ${formatDateTime(row.measured_at)}`}
              emptyMessage="Aucune donnée"
            />
          )}
        </CardContent>
      </Card>

      <ManualWeightForm open={formOpen} onOpenChange={setFormOpen} />
    </div>
  );
}
