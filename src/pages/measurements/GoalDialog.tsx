import { useId } from "react";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { DIRECTION_OPTIONS } from "@/lib/constants";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";

interface Props {
  goalDialogOpen: boolean;
  setGoalDialogOpen: (open: boolean) => void;
  goalValue: string;
  setGoalValue: (v: string) => void;
  goalDirection: string;
  setGoalDirection: (v: string) => void;
  handleDeleteGoal: () => void;
  handleSaveGoal: () => void;
}

export function GoalDialog({
  goalDialogOpen,
  setGoalDialogOpen,
  goalValue,
  setGoalValue,
  goalDirection,
  setGoalDirection,
  handleDeleteGoal,
  handleSaveGoal,
}: Props) {
  const valueId = useId();
  const directionId = useId();
  return (
    <Dialog open={goalDialogOpen} onOpenChange={setGoalDialogOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Définir un objectif</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor={valueId}>Valeur cible</Label>
            <Input
              id={valueId}
              type="number"
              value={goalValue}
              onChange={(e) => setGoalValue(e.target.value)}
              placeholder="Ex: 75"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor={directionId}>Direction</Label>
            <Select
              id={directionId}
              value={goalDirection}
              onChange={(e) => setGoalDirection(e.target.value)}
            >
              {DIRECTION_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </Select>
          </div>
        </div>
        <DialogFooter>
          <Button variant="ghost" onClick={handleDeleteGoal}>
            Supprimer
          </Button>
          <Button onClick={handleSaveGoal}>Enregistrer</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
