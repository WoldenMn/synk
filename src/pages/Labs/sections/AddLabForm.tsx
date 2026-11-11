import { Check } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import type { LabDraft, LabDraftErrors } from "../labDraft";
import { Input } from "@/components/ui/input";

interface Props {
  draft: LabDraft;
  errors: LabDraftErrors;
  onChange: (d: LabDraft) => void;
  onCancel: () => void;
  onSubmit: () => void;
}

const FIELD = "w-full rounded-md border bg-background px-3 py-2 text-sm";

function FieldError({ id, message }: { id: string; message?: string }) {
  if (!message) return null;
  return (
    <p id={id} className="text-xs text-destructive-text">
      {message}
    </p>
  );
}

function invalid(id: string, message?: string) {
  return {
    "aria-invalid": message ? true : undefined,
    "aria-describedby": message ? id : undefined,
  } as const;
}

export function AddLabForm({ draft, errors, onChange, onCancel, onSubmit }: Props) {
  return (
    <Card>
      <CardContent className="pt-6 space-y-3">
        <div className="grid gap-3 grid-cols-3">
          <div className="space-y-1">
            <label htmlFor="lab-date" className="text-sm font-medium">
              Date
            </label>
            <Input
              id="lab-date"
              type="date"
              value={draft.date}
              onChange={(e) => onChange({ ...draft, date: e.target.value })}
              className={FIELD}
              {...invalid("lab-date-error", errors.date)}
            />
            <FieldError id="lab-date-error" message={errors.date} />
          </div>
          <div className="space-y-1">
            <label htmlFor="lab-test-name" className="text-sm font-medium">
              Nom de l'analyse *
            </label>
            <Input
              id="lab-test-name"
              type="text"
              value={draft.test_name}
              onChange={(e) => onChange({ ...draft, test_name: e.target.value })}
              placeholder="Ex: Glycémie, HDL, TSH..."
              className={FIELD}
              {...invalid("lab-test-name-error", errors.test_name)}
            />
            <FieldError id="lab-test-name-error" message={errors.test_name} />
          </div>
          <div className="space-y-1">
            <label htmlFor="lab-value" className="text-sm font-medium">
              Valeur *
            </label>
            <Input
              id="lab-value"
              type="number"
              step="any"
              value={draft.value}
              onChange={(e) => onChange({ ...draft, value: e.target.value })}
              className={FIELD}
              {...invalid("lab-value-error", errors.value)}
            />
            <FieldError id="lab-value-error" message={errors.value} />
          </div>
          <div className="space-y-1">
            <label htmlFor="lab-unit" className="text-sm font-medium">
              Unité
            </label>
            <Input
              id="lab-unit"
              type="text"
              value={draft.unit}
              onChange={(e) => onChange({ ...draft, unit: e.target.value })}
              placeholder="Ex: g/L, mmol/L, mUI/L..."
              className={FIELD}
            />
          </div>
          <div className="space-y-1">
            <label htmlFor="lab-ref-min" className="text-sm font-medium">
              Ref. min
            </label>
            <Input
              id="lab-ref-min"
              type="number"
              step="any"
              value={draft.reference_min}
              onChange={(e) => onChange({ ...draft, reference_min: e.target.value })}
              className={FIELD}
              {...invalid("lab-ref-min-error", errors.reference_min)}
            />
            <FieldError id="lab-ref-min-error" message={errors.reference_min} />
          </div>
          <div className="space-y-1">
            <label htmlFor="lab-ref-max" className="text-sm font-medium">
              Ref. max
            </label>
            <Input
              id="lab-ref-max"
              type="number"
              step="any"
              value={draft.reference_max}
              onChange={(e) => onChange({ ...draft, reference_max: e.target.value })}
              className={FIELD}
              {...invalid("lab-ref-max-error", errors.reference_max)}
            />
            <FieldError id="lab-ref-max-error" message={errors.reference_max} />
          </div>
        </div>
        <div className="space-y-1">
          <label htmlFor="lab-notes" className="text-sm font-medium">
            Notes
          </label>
          <Input
            id="lab-notes"
            type="text"
            value={draft.notes}
            onChange={(e) => onChange({ ...draft, notes: e.target.value })}
            className={FIELD}
          />
        </div>
        <div className="flex gap-2 justify-end">
          <Button variant="outline" size="sm" onClick={onCancel}>
            Annuler
          </Button>
          <Button size="sm" onClick={onSubmit}>
            <Check className="mr-2 h-4 w-4" />
            Enregistrer
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
