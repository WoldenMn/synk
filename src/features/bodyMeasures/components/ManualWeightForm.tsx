import { createManualForm } from "@/components/forms/manual/createManualForm";
import { addManualBodyMeasure } from "../api";
import type { ManualBodyMeasureInput } from "../types";

interface ManualWeightInput extends Record<string, unknown> {
  measured_at: string;
  weight: number | null;
  fat_ratio: number | null;
  muscle_mass: number | null;
  bone_mass: number | null;
  hydration: number | null;
}

export const ManualWeightForm = createManualForm<ManualWeightInput>({
  title: "Ajouter une pesée",
  description: "Saisissez vos mesures de composition corporelle",
  fields: [
    { name: "measured_at", label: "Date et heure", type: "datetime", required: true },
    {
      name: "weight",
      label: "Poids",
      type: "decimal",
      unit: "kg",
      decimals: 1,
      min: 0,
      placeholder: "75.0",
      required: true,
    },
    {
      name: "fat_ratio",
      label: "Masse grasse",
      type: "decimal",
      unit: "%",
      decimals: 1,
      min: 0,
      max: 100,
      placeholder: "20.0",
    },
    {
      name: "muscle_mass",
      label: "Masse musculaire",
      type: "decimal",
      unit: "kg",
      decimals: 1,
      min: 0,
      placeholder: "35.0",
    },
    {
      name: "bone_mass",
      label: "Masse osseuse",
      type: "decimal",
      unit: "kg",
      decimals: 1,
      min: 0,
      placeholder: "3.0",
    },
    {
      name: "hydration",
      label: "Hydratation",
      type: "decimal",
      unit: "kg",
      decimals: 1,
      min: 0,
      placeholder: "40.0",
    },
  ],
  submit: async (data) => {
    const payload: ManualBodyMeasureInput = {
      measured_at: data.measured_at,
      weight: data.weight,
      fat_ratio: data.fat_ratio,
      fat_mass: null,
      muscle_mass: data.muscle_mass,
      bone_mass: data.bone_mass,
      hydration: data.hydration,
    };
    return addManualBodyMeasure(payload);
  },
  successMessage: "Pesée enregistrée",
  invalidateKeys: ["bodyMeasures"],
});
