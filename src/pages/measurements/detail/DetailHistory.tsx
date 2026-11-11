import { Ruler } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { DataTable } from "@/components/shared/DataTable";
import { EmptyState } from "@/components/shared/EmptyState";
import { LoadingSkeleton } from "@/components/shared/LoadingSkeleton";
import { formatDateTime, formatNumber } from "@/lib/formatters";
import type { ManualMeasurement, MeasurementType } from "@/features/manualMeasurements/types";

interface Props {
  measurements: ManualMeasurement[] | undefined;
  isLoading: boolean;
  selectedType: MeasurementType | undefined;
  onDelete: (id: number) => void;
}

export function DetailHistory({ measurements, isLoading, selectedType, onDelete }: Props) {
  const columns = [
    {
      key: "measured_at",
      label: "Date",
      render: (row: ManualMeasurement) => formatDateTime(row.measured_at),
      searchText: (row: ManualMeasurement) => formatDateTime(row.measured_at),
    },
    {
      key: "value",
      label: `Valeur (${selectedType?.unit || ""})`,
      render: (row: ManualMeasurement) => formatNumber(row.value),
      searchText: (row: ManualMeasurement) => formatNumber(row.value),
    },
    {
      key: "notes",
      label: "Notes",
      searchText: (row: ManualMeasurement) => row.notes ?? "",
      render: (row: ManualMeasurement) => (
        <span className="text-muted-foreground">{row.notes || "\u2014"}</span>
      ),
    },
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3">Historique</CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <LoadingSkeleton />
        ) : !measurements || measurements.length === 0 ? (
          <EmptyState
            icon={Ruler}
            title="Aucune donnée"
            description="Ajoutez votre première mensuration pour ce type"
          />
        ) : (
          <DataTable
            label={`Historique : ${selectedType?.label ?? "mensuration"}`}
            columns={columns}
            data={measurements}
            onDelete={onDelete}
            rowLabel={(row) => `la mensuration du ${formatDateTime(row.measured_at)}`}
            emptyMessage="Aucune donnée"
          />
        )}
      </CardContent>
    </Card>
  );
}
