import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Target, Plus, Trash } from "lucide-react";
import { useGoals, useDeleteGoal } from "@/features/goals/hooks";
import { GoalForm } from "@/components/forms/GoalForm";
import { formatNumber } from "@/lib/formatters";
import { DIRECTION_LABELS } from "@/lib/constants";
import { useConfirm } from "@/components/ui/confirm-dialog";

const METRIC_LABELS: Record<string, string> = {
  weight: "Poids (kg)",
  fat_ratio: "Masse grasse (%)",
  muscle_mass: "Masse musculaire (kg)",
  systolic: "Systolique (mmHg)",
  diastolic: "Diastolique (mmHg)",
  heart_rate: "Fréquence cardiaque (bpm)",
  steps: "Pas quotidiens",
  calories: "Calories actives",
  sleep_score: "Score de sommeil",
};

export function GoalsCard() {
  const { data: goals } = useGoals();
  const deleteGoal = useDeleteGoal();
  const confirm = useConfirm();
  const [goalFormOpen, setGoalFormOpen] = useState(false);

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0">
          <div className="space-y-1.5">
            <CardTitle as="h3" className="flex items-center gap-2">
              <Target className="h-5 w-5" />
              Objectifs
            </CardTitle>
            <CardDescription>Définissez vos objectifs de santé</CardDescription>
          </div>
          <Button variant="outline" size="sm" onClick={() => setGoalFormOpen(true)}>
            <Plus className="mr-2 h-4 w-4" />
            Ajouter
          </Button>
        </CardHeader>
        <CardContent>
          {!goals || goals.length === 0 ? (
            <p className="text-sm text-muted-foreground py-4 text-center">Aucun objectif défini</p>
          ) : (
            <div className="space-y-3">
              {goals.map((goal) => (
                <div
                  key={goal.metric_key}
                  className="flex items-center justify-between rounded-lg border p-3"
                >
                  <div className="space-y-0.5">
                    <p className="text-sm font-medium">
                      {METRIC_LABELS[goal.metric_key] || goal.metric_key}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {DIRECTION_LABELS[goal.direction] || goal.direction} {"\u2192 "}{" "}
                      {formatNumber(goal.target_value)}
                    </p>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 text-muted-foreground hover:text-destructive-text"
                    onClick={async () => {
                      const label = METRIC_LABELS[goal.metric_key] || goal.metric_key;
                      if (
                        !(await confirm({
                          title: `Supprimer l'objectif "${label}" ?`,
                          destructive: true,
                        }))
                      )
                        return;
                      deleteGoal.mutate(goal.metric_key);
                    }}
                    aria-label={`Supprimer l'objectif ${METRIC_LABELS[goal.metric_key] || goal.metric_key}`}
                  >
                    <Trash className="h-4 w-4" />
                  </Button>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <GoalForm open={goalFormOpen} onOpenChange={setGoalFormOpen} />
    </>
  );
}
