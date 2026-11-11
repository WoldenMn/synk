import { createCrudHook } from "@/hooks/createCrudHook";
import { getSleepRecords, getLatestSleep, addManualSleep, deleteSleepRecord } from "./api";
import type { SleepRecord, ManualSleepInput } from "./types";

const hooks = createCrudHook<SleepRecord, ManualSleepInput>({
  domain: "sleep",
  list: getSleepRecords,
  latest: getLatestSleep,
  add: addManualSleep,
  remove: deleteSleepRecord,
  toasts: {
    addSuccess: "Sommeil ajouté",
    deleteSuccess: "Sommeil supprimé",
  },
});

export const useSleepData = hooks.useList;
export const useLatestSleep = hooks.useLatest;
export const useAddSleep = hooks.useAdd;
export const useDeleteSleep = hooks.useDelete;
