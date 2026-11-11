import { createCrudHook } from "@/hooks/createCrudHook";
import { getWorkouts, getLatestWorkout, addManualWorkout, deleteWorkout } from "./api";
import type { Workout, ManualWorkoutInput } from "./types";

const hooks = createCrudHook<Workout, ManualWorkoutInput>({
  domain: "workouts",
  list: getWorkouts,
  latest: getLatestWorkout,
  add: addManualWorkout,
  remove: deleteWorkout,
  staleTime: 5 * 60 * 1000,
  toasts: {
    addSuccess: "Entraînement ajouté",
    deleteSuccess: "Entraînement supprimé",
  },
});

export const useWorkoutsData = hooks.useList;
export const useLatestWorkout = hooks.useLatest;
export const useAddWorkout = hooks.useAdd;
export const useDeleteWorkout = hooks.useDelete;
