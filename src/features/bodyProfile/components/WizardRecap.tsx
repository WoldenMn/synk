import { findMeasurement } from "../catalog";

interface WizardRecapProps {
  values: Record<string, string>;
}

export function WizardRecap({ values }: WizardRecapProps) {
  const filled = Object.entries(values).filter(([, v]) => v.trim() !== "");

  if (filled.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        Aucune mesure saisie. Revenez en arrière pour remplir au moins un champ avant d'enregistrer.
      </p>
    );
  }

  return (
    <div className="space-y-3">
      <p className="text-sm text-muted-foreground">
        {filled.length === 0
          ? "Aucune mesure saisie. Revenez en arrière pour en renseigner au moins une."
          : `${filled.length} mesure${filled.length > 1 ? "s" : ""} a enregistrer. Les champs vides sont ignorés.`}
      </p>
      <div className="max-h-80 overflow-y-auto rounded-lg border [&>:not([hidden])~:not([hidden])]:border-t">
        {filled.map(([slug, value]) => (
          <div key={slug} className="flex items-center justify-between px-3 py-2 text-sm">
            <span>{findMeasurement(slug)?.label ?? slug}</span>
            <span className="font-medium">{value} cm</span>
          </div>
        ))}
      </div>
    </div>
  );
}
