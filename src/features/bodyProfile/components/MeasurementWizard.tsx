import { useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { buildSteps, type Course } from "../steps";
import { useAddMeasurementSession, useBodySheet } from "../hooks";
import { WizardStepView } from "./WizardStepView";
import { WizardRecap } from "./WizardRecap";
import type { SessionEntry } from "../types";
import { nowDate } from "@/lib/dates";

interface MeasurementWizardProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function MeasurementWizard({ open, onOpenChange }: MeasurementWizardProps) {
  const [course, setCourse] = useState<Course | null>(null);
  const [index, setIndex] = useState(0);
  const [recap, setRecap] = useState(false);
  const [values, setValues] = useState<Record<string, string>>({});
  const [lastKnown, setLastKnown] = useState<Record<string, string>>({});

  const { data: sheet } = useBodySheet();
  const addSession = useAddMeasurementSession();
  const confirm = useConfirm();
  const steps = useMemo(() => (course ? buildSteps(course) : []), [course]);
  const currentStep = steps[index];
  const hasAnyValue = Object.values(values).some((v) => v.trim() !== "");

  function start(next: Course) {
    const known: Record<string, string> = {};
    for (const row of sheet ?? []) {
      if (row.value != null) known[row.slug] = String(row.value);
    }
    setLastKnown(known);
    setValues({});
    setIndex(0);
    setRecap(false);
    setCourse(next);
  }

  function close() {
    setCourse(null);
    setIndex(0);
    setRecap(false);
    setValues({});
    setLastKnown({});
    onOpenChange(false);
  }

  async function requestClose() {
    if (hasAnyValue) {
      const ok = await confirm({
        title: "Abandonner ce relevé ?",
        description: "Les mesures saisies n'ont pas été enregistrées et seront perdues.",
        confirmLabel: "Abandonner",
        cancelLabel: "Continuer la saisie",
        destructive: true,
      });
      if (!ok) return;
    }
    close();
  }

  function save() {
    const entries: SessionEntry[] = Object.entries(values)
      .filter(([, v]) => v.trim() !== "" && Number.isFinite(Number(v)))
      .map(([slug, v]) => ({ slug, value: Number(v) }));
    if (entries.length === 0) return;
    addSession.mutate({ measuredAt: nowDate(), entries }, { onSuccess: close });
  }

  return (
    <Dialog open={open} onOpenChange={(v) => (v ? onOpenChange(true) : void requestClose())}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>Nouveau relevé</DialogTitle>
          <DialogDescription>
            {course === null
              ? "Choisissez la profondeur du relevé."
              : recap
                ? "Vérifiez avant d'enregistrer."
                : `${index + 1} / ${steps.length}`}
          </DialogDescription>
        </DialogHeader>

        {course === null ? (
          <div className="grid grid-cols-2 gap-3 py-4">
            <Button variant="outline" className="h-24 flex-col" onClick={() => start("essential")}>
              <span className="font-semibold">Essentiel</span>
              <span className="text-xs text-muted-foreground">16 étapes</span>
            </Button>
            <Button variant="outline" className="h-24 flex-col" onClick={() => start("full")}>
              <span className="font-semibold">Complet</span>
              <span className="text-xs text-muted-foreground">33 étapes</span>
            </Button>
          </div>
        ) : recap ? (
          <div className="py-4">
            <WizardRecap values={values} />
          </div>
        ) : (
          <div className="py-4">
            <div className="mb-4 flex gap-0.5" role="group" aria-label="Progression du relevé">
              {steps.map((s, i) => (
                <button
                  key={s.id}
                  type="button"
                  aria-label={`Aller à l'étape ${i + 1} : ${s.label}`}
                  aria-current={i === index ? "step" : undefined}
                  onClick={() => setIndex(i)}
                  className={`h-1.5 flex-1 rounded-full transition-colors ${
                    i <= index ? "bg-primary" : "bg-muted"
                  }`}
                />
              ))}
            </div>
            {currentStep && (
              <WizardStepView
                step={currentStep}
                values={values}
                lastKnown={lastKnown}
                onChange={(slug, value) => setValues((v) => ({ ...v, [slug]: value }))}
                onAdvance={() => setIndex((i) => Math.min(steps.length - 1, i + 1))}
              />
            )}
          </div>
        )}

        {course !== null && (
          <DialogFooter>
            {recap ? (
              <>
                <Button variant="outline" onClick={() => setRecap(false)}>
                  Retour
                </Button>
                <Button onClick={save} disabled={addSession.isPending || !hasAnyValue}>
                  Enregistrer
                </Button>
              </>
            ) : (
              <>
                <Button
                  variant="ghost"
                  onClick={() => setIndex((i) => Math.max(0, i - 1))}
                  disabled={index === 0}
                >
                  Précédent
                </Button>
                {index < steps.length - 1 ? (
                  <Button onClick={() => setIndex((i) => i + 1)}>Suivant</Button>
                ) : null}
                <Button variant="secondary" onClick={() => setRecap(true)}>
                  Recapitulatif
                </Button>
              </>
            )}
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  );
}
