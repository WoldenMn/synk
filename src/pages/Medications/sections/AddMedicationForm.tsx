import { Check } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import type { AddMedicationInput } from "@/features/medications/types";
import { Input } from "@/components/ui/input";

interface AddMedicationFormProps {
  formData: AddMedicationInput;
  onFormDataChange: (data: AddMedicationInput) => void;
  onCancel: () => void;
  onSubmit: () => void;
}

export function AddMedicationForm({
  formData,
  onFormDataChange,
  onCancel,
  onSubmit,
}: AddMedicationFormProps) {
  return (
    <Card>
      <CardContent className="pt-6 space-y-3">
        <div className="grid gap-3 sm:grid-cols-2">
          <div className="space-y-1">
            <label htmlFor="med-name" className="text-sm font-medium">
              Nom *
            </label>
            <Input
              id="med-name"
              type="text"
              value={formData.name}
              onChange={(e) => onFormDataChange({ ...formData, name: e.target.value })}
              placeholder="Ex: Doliprane"
              className="w-full rounded-md border bg-background px-3 py-2 text-sm"
            />
          </div>
          <div className="space-y-1">
            <label htmlFor="med-dosage" className="text-sm font-medium">
              Dosage
            </label>
            <Input
              id="med-dosage"
              type="text"
              value={formData.dosage ?? ""}
              onChange={(e) => onFormDataChange({ ...formData, dosage: e.target.value || null })}
              placeholder="Ex: 1000mg"
              className="w-full rounded-md border bg-background px-3 py-2 text-sm"
            />
          </div>
          <div className="space-y-1">
            <label htmlFor="med-frequency" className="text-sm font-medium">
              Fréquence
            </label>
            <Input
              id="med-frequency"
              type="text"
              value={formData.frequency ?? ""}
              onChange={(e) => onFormDataChange({ ...formData, frequency: e.target.value || null })}
              placeholder="Ex: 2x/jour"
              className="w-full rounded-md border bg-background px-3 py-2 text-sm"
            />
          </div>
          <div className="space-y-1">
            <label htmlFor="med-notes" className="text-sm font-medium">
              Notes
            </label>
            <Input
              id="med-notes"
              type="text"
              value={formData.notes ?? ""}
              onChange={(e) => onFormDataChange({ ...formData, notes: e.target.value || null })}
              className="w-full rounded-md border bg-background px-3 py-2 text-sm"
            />
          </div>
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
