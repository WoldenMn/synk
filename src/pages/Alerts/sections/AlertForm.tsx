import { Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { METRIC_OPTIONS, CONDITION_OPTIONS } from "../constants";
import type { AlertDraft, AlertDraftErrors } from "../alertDraft";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";

interface AlertFormProps {
  draft: AlertDraft;
  errors: AlertDraftErrors;
  onDraftChange: (data: AlertDraft) => void;
  onToggleForm: () => void;
  onAdd: () => void;
}

export function AlertForm({ draft, errors, onDraftChange, onToggleForm, onAdd }: AlertFormProps) {
  return (
    <div className="rounded-md border p-4 space-y-3 bg-muted/50">
      <div className="grid gap-3 sm:grid-cols-2">
        <div className="space-y-1">
          <label htmlFor="alert-metric" className="text-sm font-medium">
            Métrique
          </label>
          <Select
            id="alert-metric"
            value={draft.metric_key}
            onChange={(e) => onDraftChange({ ...draft, metric_key: e.target.value })}
            className="w-full rounded-md border bg-background px-3 py-2 text-sm"
          >
            {METRIC_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </Select>
        </div>
        <div className="space-y-1">
          <label htmlFor="alert-condition" className="text-sm font-medium">
            Condition
          </label>
          <Select
            id="alert-condition"
            value={draft.condition}
            onChange={(e) => onDraftChange({ ...draft, condition: e.target.value })}
            className="w-full rounded-md border bg-background px-3 py-2 text-sm"
          >
            {CONDITION_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </Select>
        </div>
        <div className="space-y-1">
          <label htmlFor="alert-threshold" className="text-sm font-medium">
            Seuil
          </label>
          <Input
            id="alert-threshold"
            type="number"
            step="any"
            value={draft.threshold}
            onChange={(e) => onDraftChange({ ...draft, threshold: e.target.value })}
            className="w-full rounded-md border bg-background px-3 py-2 text-sm"
            aria-invalid={errors.threshold ? true : undefined}
            aria-describedby={errors.threshold ? "alert-threshold-error" : undefined}
          />
          {errors.threshold && (
            <p id="alert-threshold-error" className="text-xs text-destructive-text">
              {errors.threshold}
            </p>
          )}
        </div>
        <div className="space-y-1">
          <label htmlFor="alert-label" className="text-sm font-medium">
            Label
          </label>
          <Input
            id="alert-label"
            type="text"
            value={draft.label}
            onChange={(e) => onDraftChange({ ...draft, label: e.target.value })}
            placeholder="Ex. : poids trop élevé"
            className="w-full rounded-md border bg-background px-3 py-2 text-sm"
            aria-invalid={errors.label ? true : undefined}
            aria-describedby={errors.label ? "alert-label-error" : undefined}
          />
          {errors.label && (
            <p id="alert-label-error" className="text-xs text-destructive-text">
              {errors.label}
            </p>
          )}
        </div>
      </div>
      <div className="flex gap-2 justify-end">
        <Button variant="outline" size="sm" onClick={onToggleForm}>
          Annuler
        </Button>
        <Button size="sm" onClick={onAdd}>
          <Check className="mr-2 h-4 w-4" />
          Enregistrer
        </Button>
      </div>
    </div>
  );
}
