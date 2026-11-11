import { createManualForm } from "@/components/forms/manual/createManualForm";
import { addManualActivity } from "../api";
import type { ManualActivityInput } from "../types";

interface ManualActivityFormInput extends Record<string, unknown> {
  date: string;
  steps: number | null;
  distance_km: number | null;
  calories_active: number | null;
}

export const ManualActivityForm = createManualForm<ManualActivityFormInput>({
  title: "Ajouter une activité",
  description: "Saisissez vos données d'activité manuellement",
  fields: [
    { name: "date", label: "Date", type: "date", required: true },
    {
      name: "steps",
      label: "Pas",
      type: "number",
      min: 0,
      placeholder: "10000",
    },
    {
      name: "distance_km",
      label: "Distance",
      type: "decimal",
      unit: "km",
      decimals: 1,
      min: 0,
      placeholder: "5.0",
    },
    {
      name: "calories_active",
      label: "Calories actives",
      type: "number",
      min: 0,
      placeholder: "300",
    },
  ],
  submit: async (data) => {
    const payload: ManualActivityInput = {
      date: data.date,
      steps: data.steps,
      distance: data.distance_km != null ? data.distance_km * 1000 : null,
      calories_active: data.calories_active,
      duration_active: null,
    };
    return addManualActivity(payload);
  },
  successMessage: "Activité enregistrée",
  invalidateKeys: ["activity"],
});
