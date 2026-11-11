import { createCrudHook } from "@/hooks/createCrudHook";
import {
  getBloodPressure,
  getLatestBloodPressure,
  addManualBloodPressure,
  deleteBloodPressure,
} from "./api";
import type { BloodPressure, ManualBloodPressureInput } from "./types";

const hooks = createCrudHook<BloodPressure, ManualBloodPressureInput>({
  domain: "bloodPressure",
  list: getBloodPressure,
  latest: getLatestBloodPressure,
  add: addManualBloodPressure,
  remove: deleteBloodPressure,
  toasts: {
    addSuccess: "Tension ajoutée avec succès",
  },
});

export const useBloodPressureData = hooks.useList;
export const useLatestBloodPressure = hooks.useLatest;
export const useAddBloodPressure = hooks.useAdd;
export const useDeleteBloodPressure = hooks.useDelete;
