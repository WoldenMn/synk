import { Pill, Trash, Check, SkipForward } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { LoadingSkeleton } from "@/components/shared";
import { format, parseISO } from "date-fns";
import type { Medication, MedicationLog } from "@/features/medications/types";
import { nowDate } from "@/lib/dates";

interface ActiveMedicationsListProps {
  medications: Medication[];
  allLogs: MedicationLog[];
  loading: boolean;
  onDelete: (id: number) => void;
  onLogTaken: (medicationId: number, skipped: boolean) => void;
}

export function ActiveMedicationsList({
  medications,
  allLogs,
  loading,
  onDelete,
  onLogTaken,
}: ActiveMedicationsListProps) {
  const today = nowDate();
  const todayLogs = allLogs.filter((l) => format(parseISO(l.taken_at), "yyyy-MM-dd") === today);
  const activeMeds = medications.filter((m) => m.active);

  if (loading) return <LoadingSkeleton rows={3} height="h-20" />;

  if (activeMeds.length === 0) {
    return (
      <Card>
        <CardContent className="py-8 text-center text-muted-foreground">
          <Pill className="mx-auto h-10 w-10 mb-2 opacity-40" />
          <p>Aucun médicament actif</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="grid gap-4 md:grid-cols-2">
      {activeMeds.map((med) => {
        const medTodayLogs = todayLogs.filter((l) => l.medication_id === med.id);
        const takenToday = medTodayLogs.some((l) => !l.skipped);
        const skippedToday = medTodayLogs.some((l) => l.skipped);

        return (
          <Card key={med.id}>
            <CardContent className="pt-6">
              <div className="flex items-start justify-between">
                <div>
                  <h3 className="font-semibold">{med.name}</h3>
                  {med.dosage && <p className="text-sm text-muted-foreground">{med.dosage}</p>}
                  {med.frequency && (
                    <p className="text-xs text-muted-foreground">{med.frequency}</p>
                  )}
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 text-destructive-text"
                  onClick={() => onDelete(med.id)}
                  aria-label={`Supprimer le médicament ${med.name}`}
                >
                  <Trash className="h-4 w-4" />
                </Button>
              </div>
              <div className="flex gap-2 mt-3">
                <Button
                  size="sm"
                  variant={takenToday ? "secondary" : "default"}
                  disabled={takenToday}
                  onClick={() => onLogTaken(med.id, false)}
                >
                  <Check className="mr-1 h-3 w-3" />
                  {takenToday ? "Prise OK" : "Prise aujourd'hui"}
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={skippedToday}
                  onClick={() => onLogTaken(med.id, true)}
                >
                  <SkipForward className="mr-1 h-3 w-3" />
                  Sauter
                </Button>
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}
