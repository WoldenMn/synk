import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { DataTable } from "@/components/shared";
import { formatDate } from "@/lib/formatters";
import type { Medication, MedicationLog } from "@/features/medications/types";

interface MedicationLogsTableProps {
  medications: Medication[];
  allLogs: MedicationLog[];
}

export function MedicationLogsTable({ medications, allLogs }: MedicationLogsTableProps) {
  if (allLogs.length === 0) return null;

  const logColumns = [
    {
      key: "taken_at",
      label: "Date",
      render: (row: MedicationLog) => formatDate(row.taken_at.substring(0, 10)),
      searchText: (row: MedicationLog) => formatDate(row.taken_at.substring(0, 10)),
    },
    {
      key: "medication_id",
      label: "Médicament",
      searchText: (row: MedicationLog) =>
        medications.find((m) => m.id === row.medication_id)?.name ?? `#${row.medication_id}`,
      render: (row: MedicationLog) =>
        medications.find((m) => m.id === row.medication_id)?.name ?? `#${row.medication_id}`,
    },
    {
      key: "skipped",
      label: "Statut",
      searchText: (row: MedicationLog) => (row.skipped ? "Sautée" : "Prise"),
      render: (row: MedicationLog) =>
        row.skipped ? (
          <span className="text-warning-text font-medium">Sautée</span>
        ) : (
          <span className="text-success-text font-medium">Prise</span>
        ),
    },
    {
      key: "notes",
      label: "Notes",
      render: (row: MedicationLog) => row.notes ?? "—",
      searchText: (row: MedicationLog) => row.notes ?? "",
    },
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle>Historique des prises</CardTitle>
      </CardHeader>
      <CardContent>
        <DataTable
          label="Historique des prises"
          columns={logColumns}
          data={allLogs}
          emptyMessage="Aucun historique"
        />
      </CardContent>
    </Card>
  );
}
