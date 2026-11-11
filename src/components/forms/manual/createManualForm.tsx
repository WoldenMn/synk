import { useState, useId } from "react";
import { toast } from "sonner";
import { useQueryClient } from "@tanstack/react-query";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { reportError } from "@/lib/reportError";
import type { FormConfig } from "./types";
import { buildInitialState, parseFieldValue, validateAll } from "./createManualForm/helpers";
import { FormFieldRenderer } from "./createManualForm/FormFieldRenderer";

export interface ManualFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function createManualForm<TInput extends Record<string, unknown>>(
  config: FormConfig<TInput>
) {
  function ManualForm({ open, onOpenChange }: ManualFormProps) {
    const [values, setValues] = useState<Record<string, string>>(() =>
      buildInitialState<TInput>(config.fields, config.defaultValues)
    );
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
    const [wasOpen, setWasOpen] = useState(open);
    if (open !== wasOpen) {
      setWasOpen(open);
      if (open) {
        setValues(buildInitialState<TInput>(config.fields, config.defaultValues));
        setError(null);
        setFieldErrors({});
      }
    }
    const formId = useId();
    const qc = useQueryClient();

    function setFieldValue(name: string, raw: string) {
      setValues((prev) => ({ ...prev, [name]: raw }));
    }

    function resetForm() {
      setValues(buildInitialState<TInput>(config.fields, config.defaultValues));
      setError(null);
      setFieldErrors({});
    }

    async function handleSubmit(e: React.FormEvent) {
      e.preventDefault();
      setError(null);

      const found = validateAll(config.fields, values);
      setFieldErrors(found);
      const firstInvalid = config.fields.find((f) => found[f.name]);
      if (firstInvalid) {
        document.getElementById(`${formId}-${firstInvalid.name}`)?.focus();
        return;
      }

      const payload: Record<string, unknown> = {};
      for (const field of config.fields) {
        const raw = values[field.name] ?? "";
        payload[field.name] = parseFieldValue(field, raw);
      }

      setSubmitting(true);
      try {
        await config.submit(payload as TInput);
        toast.success(config.successMessage ?? "Enregistré");
        if (config.invalidateKeys) {
          for (const key of config.invalidateKeys) {
            qc.invalidateQueries({ queryKey: [key] });
          }
        }
        if (config.onSuccess) {
          config.onSuccess();
        }
        resetForm();
        onOpenChange(false);
      } catch (err: unknown) {
        const { userMessage } = reportError(err, {
          source: "manual",
          context: config.title ?? "form",
        });
        setError(userMessage);
      } finally {
        setSubmitting(false);
      }
    }

    return (
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{config.title ?? "Ajouter une mesure"}</DialogTitle>
            {config.description ? (
              <DialogDescription>{config.description}</DialogDescription>
            ) : null}
          </DialogHeader>
          <form onSubmit={handleSubmit} className="space-y-4">
            {config.fields.map((field) => (
              <FormFieldRenderer
                key={field.name}
                field={field}
                value={values[field.name] ?? ""}
                inputId={`${formId}-${field.name}`}
                error={fieldErrors[field.name]}
                onChange={(raw) => setFieldValue(field.name, raw)}
              />
            ))}

            {error ? (
              <p role="alert" className="text-sm text-destructive-text">
                {error}
              </p>
            ) : null}

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={submitting}
              >
                {config.cancelLabel ?? "Annuler"}
              </Button>
              <Button type="submit" disabled={submitting}>
                {submitting ? "Enregistrement..." : (config.submitLabel ?? "Enregistrer")}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    );
  }
  ManualForm.displayName = "ManualForm";
  return ManualForm;
}
