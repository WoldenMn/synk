import { createManualForm } from "@/components/forms/manual/createManualForm";
import { addManualWorkout } from "../api";
import type { ManualWorkoutInput } from "../types";

const WORKOUT_CATEGORIES = [
  "Course",
  "Marche",
  "Vélo",
  "Natation",
  "Musculation",
  "Yoga",
  "HIIT",
  "Autre",
];

interface ManualWorkoutFormInput extends Record<string, unknown> {
  date: string;
  category_label: string;
  duration_minutes: number | null;
  distance_km: number | null;
  calories: number | null;
  hr_average: number | null;
}

export const ManualWorkoutForm = createManualForm<ManualWorkoutFormInput>({
  title: "Ajouter un entraînement",
  description: "Saisissez les détails de votre entraînement",
  fields: [
    { name: "date", label: "Date", type: "date", required: true },
    {
      name: "category_label",
      label: "Type d'entraînement",
      type: "select",
      options: WORKOUT_CATEGORIES.map((c) => ({ value: c, label: c })),
    },
    {
      name: "duration_minutes",
      label: "Durée",
      type: "number",
      unit: "minutes",
      min: 0,
      placeholder: "60",
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
      name: "calories",
      label: "Calories",
      type: "number",
      min: 0,
      placeholder: "300",
    },
    {
      name: "hr_average",
      label: "FC moyenne",
      type: "number",
      unit: "bpm",
      min: 0,
      max: 250,
      placeholder: "140",
    },
  ],
  defaultValues: { category_label: "Autre" },
  submit: async (data) => {
    const payload: ManualWorkoutInput = {
      date: data.date,
      category_label: data.category_label || null,
      duration: data.duration_minutes != null ? data.duration_minutes * 60 : null,
      distance: data.distance_km != null ? data.distance_km * 1000 : null,
      steps: null,
      calories: data.calories,
      hr_average: data.hr_average,
    };
    return addManualWorkout(payload);
  },
  successMessage: "Entraînement enregistré",
  invalidateKeys: ["workouts"],
});
