import { useState } from "react";
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
import { useUpsertGoal } from "@/features/goals/hooks";
import { toast } from "sonner";

interface GoalFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const METRIC_OPTIONS = [
  { value: "weight", label: "Poids (kg)" },
  { value: "fat_ratio", label: "Masse grasse (%)" },
  { value: "muscle_mass", label: "Masse musculaire (kg)" },
  { value: "systolic", label: "Systolique (mmHg)" },
  { value: "diastolic", label: "Diastolique (mmHg)" },
  { value: "heart_rate", label: "Fréquence cardiaque (bpm)" },
  { value: "steps", label: "Pas quotidiens" },
  { value: "calories", label: "Calories actives" },
  { value: "sleep_score", label: "Score de sommeil" },
];

const DIRECTION_OPTIONS = [
  { value: "decrease", label: "Diminuer" },
  { value: "increase", label: "Augmenter" },
  { value: "maintain", label: "Maintenir" },
];

export function GoalForm({ open, onOpenChange }: GoalFormProps) {
  const [metricKey, setMetricKey] = useState("");
  const [targetValue, setTargetValue] = useState("");
  const [direction, setDirection] = useState("");

  const upsertMutation = useUpsertGoal();

  function resetForm() {
    setMetricKey("");
    setTargetValue("");
    setDirection("");
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!metricKey || !targetValue || !direction) return;

    upsertMutation.mutate(
      {
        metric_key: metricKey,
        target_value: parseFloat(targetValue),
        direction,
      },
      {
        onSuccess: () => {
          toast.success("Objectif enregistré");
          resetForm();
          onOpenChange(false);
        },
      }
    );
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Définir un objectif</DialogTitle>
          <DialogDescription>Choisissez une métrique et définissez votre cible</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="goal-metric">Métrique *</Label>
            <Select
              id="goal-metric"
              value={metricKey}
              onChange={(e) => setMetricKey(e.target.value)}
              required
            >
              <option value="">Sélectionnez une métrique</option>
              {METRIC_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="goal-target">Valeur cible *</Label>
            <Input
              id="goal-target"
              type="number"
              step="0.1"
              required
              placeholder="0.0"
              value={targetValue}
              onChange={(e) => setTargetValue(e.target.value)}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="goal-direction">Direction *</Label>
            <Select
              id="goal-direction"
              value={direction}
              onChange={(e) => setDirection(e.target.value)}
              required
            >
              <option value="">Sélectionnez une direction</option>
              {DIRECTION_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </Select>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Annuler
            </Button>
            <Button
              type="submit"
              disabled={upsertMutation.isPending || !metricKey || !targetValue || !direction}
            >
              {upsertMutation.isPending ? "Enregistrement..." : "Enregistrer"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
