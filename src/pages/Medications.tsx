import { useState } from "react";
import { toast } from "sonner";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { PageHeader } from "@/components/shared";
import { getMedicationLogs, logMedicationTaken } from "@/features/medications/api";
import {
  useMedications,
  useAddMedication,
  useDeleteMedication,
} from "@/features/medications/hooks";
import type { AddMedicationInput, Medication, MedicationLog } from "@/features/medications/types";
import { reportError, notifyError } from "@/lib/reportError";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { useTimeRange } from "@/hooks/useTimeRange";
import { MedicationStats } from "./Medications/sections/MedicationStats";
import { AddMedicationForm } from "./Medications/sections/AddMedicationForm";
import { ActiveMedicationsList } from "./Medications/sections/ActiveMedicationsList";
import { MedicationLogsTable } from "./Medications/sections/MedicationLogsTable";

const DEFAULT_FORM: AddMedicationInput = {
  name: "",
  dosage: null,
  frequency: null,
  start_date: null,
  end_date: null,
  notes: null,
};

async function fetchAllLogs(
  medications: Medication[],
  from: string,
  to: string
): Promise<MedicationLog[]> {
  const logs = await Promise.all(medications.map((m) => getMedicationLogs(m.id, from, to)));
  return logs.flat();
}

export default function Medications() {
  const { timeRange, setTimeRange, from, to } = useTimeRange();
  const qc = useQueryClient();
  const { data: medications = [], isLoading: medsLoading } = useMedications();
  const { data: allLogs = [], isLoading: logsLoading } = useQuery({
    queryKey: ["medications", "logs", from, to, medications.map((m) => m.id)],
    queryFn: () => fetchAllLogs(medications, from, to),
    enabled: medications.length > 0,
  });
  const addMed = useAddMedication();
  const deleteMed = useDeleteMedication();
  const logMed = useMutation({
    mutationFn: logMedicationTaken,
    onSuccess: (_id, vars) => {
      qc.invalidateQueries({ queryKey: ["medications"] });
      toast.success(vars.skipped ? "Prise sautée" : "Prise enregistrée");
    },
    onError: (e: unknown) => reportError(e, { source: "mutation", context: "medications" }),
  });

  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState<AddMedicationInput>(DEFAULT_FORM);
  const confirm = useConfirm();

  const loading = medsLoading || (medications.length > 0 && logsLoading);

  function handleAdd() {
    if (!formData.name.trim()) {
      notifyError("Le nom est requis");
      return;
    }
    addMed.mutate(formData, {
      onSuccess: () => {
        setShowForm(false);
        setFormData(DEFAULT_FORM);
      },
    });
  }

  async function handleDelete(id: number) {
    if (!(await confirm({ title: "Supprimer ce médicament ?", destructive: true }))) return;
    deleteMed.mutate(id);
  }

  function handleLogTaken(medicationId: number, skipped: boolean) {
    logMed.mutate({
      medication_id: medicationId,
      taken_at: new Date().toISOString(),
      skipped,
      notes: null,
    });
  }

  return (
    <div className="space-y-6">
      <PageHeader
        timeRange={timeRange}
        onTimeRangeChange={setTimeRange}
        onAdd={() => setShowForm(true)}
      />
      <MedicationStats medications={medications} allLogs={allLogs} />
      {showForm && (
        <AddMedicationForm
          formData={formData}
          onFormDataChange={setFormData}
          onCancel={() => setShowForm(false)}
          onSubmit={handleAdd}
        />
      )}
      <ActiveMedicationsList
        medications={medications}
        allLogs={allLogs}
        loading={loading}
        onDelete={handleDelete}
        onLogTaken={handleLogTaken}
      />
      <MedicationLogsTable medications={medications} allLogs={allLogs} />
    </div>
  );
}
