import type { AddAlertInput } from "@/features/alerts/types";

export interface AlertDraft {
  metric_key: string;
  condition: string;
  threshold: string;
  label: string;
}

export type AlertDraftErrors = Partial<Record<"threshold" | "label", string>>;

export function emptyAlertDraft(): AlertDraft {
  return { metric_key: "weight", condition: "above", threshold: "", label: "" };
}

export function parseAlertDraft(
  d: AlertDraft
): { ok: true; input: AddAlertInput } | { ok: false; errors: AlertDraftErrors } {
  const errors: AlertDraftErrors = {};
  const threshold = Number(d.threshold.replace(",", "."));
  if (d.threshold.trim() === "") errors.threshold = "Indiquez un seuil.";
  else if (!Number.isFinite(threshold)) errors.threshold = "Le seuil doit être un nombre.";
  if (d.label.trim() === "") errors.label = "Donnez un nom à l'alerte.";
  if (Object.keys(errors).length > 0) return { ok: false, errors };
  return {
    ok: true,
    input: { metric_key: d.metric_key, condition: d.condition, threshold, label: d.label.trim() },
  };
}
