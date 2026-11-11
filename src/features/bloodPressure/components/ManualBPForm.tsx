import { createManualForm } from "@/components/forms/manual/createManualForm";
import { addManualBloodPressure } from "../api";
import type { ManualBloodPressureInput } from "../types";

interface ManualBPInput extends Record<string, unknown> {
  measured_at: string;
  systolic: number;
  diastolic: number;
  heart_rate: number | null;
}

export const ManualBPForm = createManualForm<ManualBPInput>({
  title: "Ajouter une mesure de tension",
  description: "Saisissez vos valeurs de pression artérielle",
  fields: [
    { name: "measured_at", label: "Date et heure", type: "datetime", required: true },
    {
      name: "systolic",
      label: "Systolique",
      type: "number",
      unit: "mmHg",
      min: 50,
      max: 300,
      placeholder: "120",
      required: true,
    },
    {
      name: "diastolic",
      label: "Diastolique",
      type: "number",
      unit: "mmHg",
      min: 30,
      max: 200,
      placeholder: "80",
      required: true,
    },
    {
      name: "heart_rate",
      label: "Fréquence cardiaque",
      type: "number",
      unit: "bpm",
      min: 30,
      max: 250,
      placeholder: "72",
    },
  ],
  submit: async (data) => {
    const payload: ManualBloodPressureInput = {
      measured_at: data.measured_at,
      systolic: data.systolic,
      diastolic: data.diastolic,
      heart_rate: data.heart_rate,
    };
    return addManualBloodPressure(payload);
  },
  successMessage: "Tension enregistrée",
  invalidateKeys: ["bloodPressure"],
});
