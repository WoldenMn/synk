import { StatCard } from "@/components/shared";
import { format, parseISO } from "date-fns";
import type { Medication, MedicationLog } from "@/features/medications/types";
import { nowDate } from "@/lib/dates";

interface MedicationStatsProps {
  medications: Medication[];
  allLogs: MedicationLog[];
}

export function MedicationStats({ medications, allLogs }: MedicationStatsProps) {
  const today = nowDate();
  const todayLogs = allLogs.filter((l) => format(parseISO(l.taken_at), "yyyy-MM-dd") === today);
  const todayTaken = todayLogs.filter((l) => !l.skipped).length;
  const totalLogs = allLogs.filter((l) => !l.skipped).length;
  const totalExpected = allLogs.length || 1;
  const adherenceRate = allLogs.length > 0 ? Math.round((totalLogs / totalExpected) * 100) : 0;
  const activeMeds = medications.filter((m) => m.active);

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <StatCard label="Médicaments actifs" value={String(activeMeds.length)} />
      <StatCard label="Prises aujourd'hui" value={String(todayTaken)} />
      <StatCard label="Taux d'observance" value={`${adherenceRate} %`} />
    </div>
  );
}
