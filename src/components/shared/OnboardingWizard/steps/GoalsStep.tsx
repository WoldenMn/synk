import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Check, SkipForward } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { logWarn } from "@/lib/logger";
import { upsertGoal } from "@/features/goals/api";
import { DIRECTION_OPTIONS } from "@/lib/constants";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";

interface GoalsStepProps {
  onComplete: () => void;
  onSkip: () => void;
  titleId: string;
}

export function GoalsStep({ onComplete, onSkip, titleId }: GoalsStepProps) {
  const [weightGoal, setWeightGoal] = useState("");
  const [stepsGoal, setStepsGoal] = useState("");
  const [weightDirection, setWeightDirection] =
    useState<(typeof DIRECTION_OPTIONS)[number]["value"]>("decrease");
  const queryClient = useQueryClient();

  async function handleSaveGoals() {
    try {
      if (weightGoal) {
        await upsertGoal({
          metric_key: "weight",
          target_value: Number(weightGoal),
          direction: weightDirection,
        });
      }
      if (stepsGoal) {
        await upsertGoal({
          metric_key: "steps",
          target_value: Number(stepsGoal),
          direction: "increase",
        });
      }
      queryClient.invalidateQueries({ queryKey: ["goals"] });
    } catch (e) {
      logWarn("[GoalsStep] upsert_goal échoué (non bloquant)", e);
      toast.warning("Données non enregistrées, vous pourrez les compléter dans Paramètres.");
    }
    onComplete();
  }

  return (
    <div className="space-y-6">
      <div className="text-center space-y-2">
        <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-primary/10">
          <Check className="h-8 w-8 text-primary-text" />
        </div>
        <h2 id={titleId} className="text-xl font-bold">
          Objectifs
        </h2>
        <p className="text-sm text-muted-foreground">
          Définissez rapidement vos objectifs principaux.
        </p>
      </div>
      <div className="grid grid-cols-2 gap-3">
        <div className="space-y-1.5">
          <label htmlFor="onboarding-weight-goal" className="text-sm font-medium">
            Poids cible (kg)
          </label>
          <Input
            id="onboarding-weight-goal"
            type="number"
            min={20}
            max={300}
            step={0.1}
            placeholder="ex. 70"
            value={weightGoal}
            onChange={(e) => setWeightGoal(e.target.value)}
            className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          />
          <label htmlFor="onboarding-weight-direction" className="text-sm font-medium">
            Sens
          </label>
          <Select
            id="onboarding-weight-direction"
            value={weightDirection}
            onChange={(e) => setWeightDirection(e.target.value as typeof weightDirection)}
            className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            {DIRECTION_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </Select>
        </div>
        <div className="space-y-1.5">
          <label htmlFor="onboarding-steps-goal" className="text-sm font-medium">
            Pas / jour
          </label>
          <Input
            id="onboarding-steps-goal"
            type="number"
            min={1000}
            max={100000}
            step={500}
            placeholder="ex. 10000"
            value={stepsGoal}
            onChange={(e) => setStepsGoal(e.target.value)}
            className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          />
        </div>
      </div>
      <div className="flex gap-3">
        <Button variant="ghost" className="flex-1" onClick={onSkip}>
          <SkipForward className="mr-2 h-4 w-4" />
          Passer
        </Button>
        <Button className="flex-1" onClick={handleSaveGoals}>
          Terminer
          <Check className="ml-2 h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
