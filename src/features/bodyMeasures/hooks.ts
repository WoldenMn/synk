import { createCrudHook } from "@/hooks/createCrudHook";
import {
  getBodyMeasures,
  getLatestBodyMeasure,
  addManualBodyMeasure,
  deleteBodyMeasure,
} from "./api";
import type { BodyMeasure, ManualBodyMeasureInput } from "./types";

const hooks = createCrudHook<BodyMeasure, ManualBodyMeasureInput>({
  domain: "bodyMeasures",
  list: getBodyMeasures,
  latest: getLatestBodyMeasure,
  add: addManualBodyMeasure,
  remove: deleteBodyMeasure,
});

export const useBodyMeasures = hooks.useList;
export const useLatestBodyMeasure = hooks.useLatest;
export const useAddBodyMeasure = hooks.useAdd;
export const useDeleteBodyMeasure = hooks.useDelete;
