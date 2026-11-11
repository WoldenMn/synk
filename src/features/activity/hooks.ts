import { createCrudHook } from "@/hooks/createCrudHook";
import { getActivity, getLatestActivity, addManualActivity, deleteActivity } from "./api";
import type { Activity, ManualActivityInput } from "./types";

const hooks = createCrudHook<Activity, ManualActivityInput>({
  domain: "activity",
  list: getActivity,
  latest: getLatestActivity,
  add: addManualActivity,
  remove: deleteActivity,
  toasts: {
    addSuccess: "Activité ajoutée",
    deleteSuccess: "Activité supprimée",
  },
});

export const useActivityData = hooks.useList;
export const useLatestActivity = hooks.useLatest;
export const useAddActivity = hooks.useAdd;
export const useDeleteActivity = hooks.useDelete;
