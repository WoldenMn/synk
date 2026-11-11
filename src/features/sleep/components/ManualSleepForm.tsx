import { createManualForm } from "@/components/forms/manual/createManualForm";
import { addManualSleep } from "../api";
import type { ManualSleepInput } from "../types";

interface ManualSleepFormInput extends Record<string, unknown> {
  date: string;
  duration_minutes: number;
  sleep_score: number | null;
}

export const ManualSleepForm = createManualForm<ManualSleepFormInput>({
  title: "Ajouter un sommeil",
  description: "Saisissez vos données de sommeil manuellement",
  fields: [
    { name: "date", label: "Date", type: "date", required: true },
    {
      name: "duration_minutes",
      label: "Durée",
      type: "number",
      unit: "minutes",
      min: 0,
      placeholder: "480",
      required: true,
    },
    {
      name: "sleep_score",
      label: "Score de sommeil (0-100)",
      type: "number",
      min: 0,
      max: 100,
      placeholder: "75",
    },
  ],
  submit: async (data) => {
    const payload: ManualSleepInput = {
      date: data.date,
      start_at: null,
      end_at: null,
      duration_total: data.duration_minutes ? data.duration_minutes * 60 : null,
      sleep_score: data.sleep_score,
    };
    return addManualSleep(payload);
  },
  successMessage: "Sommeil enregistré",
  invalidateKeys: ["sleep"],
});
