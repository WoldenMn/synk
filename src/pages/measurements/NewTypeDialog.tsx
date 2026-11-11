import { useId, useState } from "react";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { useAddMeasurementType } from "@/features/manualMeasurements/hooks";

interface Props {
  newTypeDialogOpen: boolean;
  setNewTypeDialogOpen: (open: boolean) => void;
}

export function NewTypeDialog({ newTypeDialogOpen, setNewTypeDialogOpen }: Props) {
  const [newTypeLabel, setNewTypeLabel] = useState("");
  const [newTypeUnit, setNewTypeUnit] = useState("cm");
  const addTypeMutation = useAddMeasurementType();
  const nameId = useId();
  const unitId = useId();

  function handleAddType(e: React.FormEvent) {
    e.preventDefault();
    if (!newTypeLabel.trim()) return;
    const slug = newTypeLabel
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "_");
    addTypeMutation.mutate(
      { slug, label: newTypeLabel.trim(), unit: newTypeUnit },
      {
        onSuccess: () => {
          setNewTypeLabel("");
          setNewTypeUnit("cm");
          setNewTypeDialogOpen(false);
        },
      }
    );
  }

  return (
    <Dialog open={newTypeDialogOpen} onOpenChange={setNewTypeDialogOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Nouveau type de mensuration</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleAddType} className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor={nameId}>Nom *</Label>
            <Input
              id={nameId}
              value={newTypeLabel}
              onChange={(e) => setNewTypeLabel(e.target.value)}
              placeholder="Ex: Tour de bras"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor={unitId}>Unité</Label>
            <Select
              id={unitId}
              value={newTypeUnit}
              onChange={(e) => setNewTypeUnit(e.target.value)}
            >
              <option value="cm">cm</option>
              <option value="mm">mm</option>
              <option value="kg">kg</option>
              <option value="%">%</option>
              <option value="u">autre</option>
            </Select>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => setNewTypeDialogOpen(false)}>
              Annuler
            </Button>
            <Button type="submit" disabled={addTypeMutation.isPending || !newTypeLabel.trim()}>
              Créer
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
