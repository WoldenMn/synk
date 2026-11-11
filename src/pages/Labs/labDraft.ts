import type { AddLabResultInput } from "@/features/labs/types";

export interface LabDraft {
  date: string;
  test_name: string;
  value: string;
  unit: string;
  reference_min: string;
  reference_max: string;
  notes: string;
}

export type LabDraftErrors = Partial<Record<keyof LabDraft, string>>;

export function emptyLabDraft(today: string): LabDraft {
  return {
    date: today,
    test_name: "",
    value: "",
    unit: "",
    reference_min: "",
    reference_max: "",
    notes: "",
  };
}

function toNumber(raw: string): number | null | undefined {
  if (raw.trim() === "") return null;
  const n = Number(raw.replace(",", "."));
  return Number.isFinite(n) ? n : undefined;
}

function referenceErrors(
  min: number | null | undefined,
  max: number | null | undefined
): LabDraftErrors {
  const errors: LabDraftErrors = {};
  if (min === undefined) errors.reference_min = "Doit être un nombre.";
  if (max === undefined) errors.reference_max = "Doit être un nombre.";
  if (typeof min === "number" && typeof max === "number" && min > max) {
    errors.reference_max = "Doit être supérieure ou égale à la référence min.";
  }
  return errors;
}

export function parseLabDraft(
  d: LabDraft
): { ok: true; input: AddLabResultInput } | { ok: false; errors: LabDraftErrors } {
  const errors: LabDraftErrors = {};
  if (d.date === "") errors.date = "Indiquez la date du résultat.";
  if (d.test_name.trim() === "") errors.test_name = "Indiquez le nom de l'analyse.";
  const value = toNumber(d.value);
  if (value === null) errors.value = "Indiquez la valeur du résultat.";
  if (value === undefined) errors.value = "La valeur doit être un nombre.";
  const min = toNumber(d.reference_min);
  const max = toNumber(d.reference_max);
  Object.assign(errors, referenceErrors(min, max));
  if (Object.keys(errors).length > 0 || typeof value !== "number") return { ok: false, errors };
  return {
    ok: true,
    input: {
      date: d.date,
      test_name: d.test_name.trim(),
      value,
      unit: d.unit.trim() || null,
      reference_min: min ?? null,
      reference_max: max ?? null,
      notes: d.notes.trim() || null,
    },
  };
}
