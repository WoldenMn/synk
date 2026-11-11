import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { BodyFigure } from "./BodyFigure";
import { findMeasurement } from "../catalog";
import type { WizardStep } from "../steps";

interface WizardStepViewProps {
  step: WizardStep;
  values: Record<string, string>;
  lastKnown: Record<string, string>;
  onChange: (slug: string, value: string) => void;
  onAdvance: () => void;
}

export function WizardStepView({
  step,
  values,
  lastKnown,
  onChange,
  onAdvance,
}: WizardStepViewProps) {
  const slugs = step.fields.map((f) => f.slug);
  const measured = slugs.flatMap((slug) => {
    const def = findMeasurement(slug);
    return def ? [def] : [];
  });
  const firstSlug = slugs[0];
  const view =
    (firstSlug === undefined ? undefined : findMeasurement(firstSlug))?.anchor?.view ?? "front";
  const soloField = step.fields.length === 1;

  return (
    <div className="grid grid-cols-[200px_1fr] gap-6">
      <BodyFigure view={view} highlight={slugs} annotations={measured} />
      <div className="space-y-4">
        {!soloField && (
          <div>
            <h3 className="text-lg font-semibold">{step.label}</h3>
            <p className="text-sm text-muted-foreground mt-1">{step.guidance}</p>
          </div>
        )}
        <div className="grid grid-cols-2 gap-3">
          {step.fields.map((f, index) => {
            const def = findMeasurement(f.slug);
            if (!def) return null;
            const isLast = index === step.fields.length - 1;
            return (
              <div key={f.slug} className={soloField ? "col-span-2 space-y-2" : "space-y-2"}>
                <Label
                  htmlFor={`field-${f.slug}`}
                  className={soloField ? "text-lg font-semibold" : undefined}
                >
                  {def.label}
                </Label>
                {soloField && <p className="text-sm text-muted-foreground">{step.guidance}</p>}
                <Input
                  id={`field-${f.slug}`}
                  type="number"
                  inputMode="decimal"
                  step="0.1"
                  min={f.min}
                  max={f.max}
                  placeholder={lastKnown[f.slug] ?? ""}
                  value={values[f.slug] ?? ""}
                  onChange={(e) => onChange(f.slug, e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key !== "Enter") return;
                    e.preventDefault();
                    if (isLast) {
                      onAdvance();
                      return;
                    }
                    const following = step.fields[index + 1];
                    if (following === undefined) return;
                    const next = document.getElementById(`field-${following.slug}`);
                    if (next instanceof HTMLElement) next.focus();
                  }}
                />
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
