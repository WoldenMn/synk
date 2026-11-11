import { useState } from "react";
import { formatISO } from "date-fns";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import { useMeasurementTypes, useAddManualMeasurement } from "../hooks";
import { toast } from "sonner";

interface ManualMeasurementFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  defaultTypeId?: number;
}

function toLocalDateTimeValue(date: Date): string {
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function ManualMeasurementForm({
  open,
  onOpenChange,
  defaultTypeId,
}: ManualMeasurementFormProps) {
  const [typeId, setTypeId] = useState<string>(defaultTypeId ? String(defaultTypeId) : "");
  const [measuredAt, setMeasuredAt] = useState(toLocalDateTimeValue(new Date()));
  const [value, setValue] = useState("");
  const [notes, setNotes] = useState("");

  const { data: types } = useMeasurementTypes();
  const addMutation = useAddManualMeasurement();

  function resetForm() {
    if (!defaultTypeId) setTypeId("");
    setMeasuredAt(toLocalDateTimeValue(new Date()));
    setValue("");
    setNotes("");
  }

  const [wasOpen, setWasOpen] = useState(open);
  if (open !== wasOpen) {
    setWasOpen(open);
    if (open) resetForm();
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!typeId || !value) return;

    const parsedDate = new Date(measuredAt);

    addMutation.mutate(
      {
        measurement_type_id: parseInt(typeId, 10),
        measured_at: formatISO(parsedDate),
        value: parseFloat(value),
        notes: notes || null,
      },
      {
        onSuccess: () => {
          toast.success("Mensuration enregistrée");
          resetForm();
          onOpenChange(false);
        },
      }
    );
  }

  const selectedType = types?.find((t) => t.id === parseInt(typeId, 10));

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Ajouter une mensuration</DialogTitle>
          <DialogDescription>Saisissez une nouvelle mesure corporelle</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="measurement-type">Type de mensuration *</Label>
            <Select
              id="measurement-type"
              value={typeId}
              onChange={(e) => setTypeId(e.target.value)}
              required
            >
              <option value="">Sélectionnez un type</option>
              {types?.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.label} ({t.unit})
                </option>
              ))}
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="measurement-date">Date et heure</Label>
            <Input
              id="measurement-date"
              type="datetime-local"
              value={measuredAt}
              onChange={(e) => setMeasuredAt(e.target.value)}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="measurement-value">
              Valeur {selectedType ? `(${selectedType.unit})` : ""} *
            </Label>
            <Input
              id="measurement-value"
              type="number"
              step="0.1"
              min="0"
              required
              placeholder="0.0"
              value={value}
              onChange={(e) => setValue(e.target.value)}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="measurement-notes">Notes</Label>
            <Input
              id="measurement-notes"
              type="text"
              placeholder="Notes optionnelles"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
            />
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Annuler
            </Button>
            <Button type="submit" disabled={addMutation.isPending || !typeId || !value}>
              {addMutation.isPending ? "Enregistrement..." : "Enregistrer"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
