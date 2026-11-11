import { Plus, Target, TrendingUp, TrendingDown, Minus, ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { DIRECTION_LABELS } from "@/lib/constants";
import type { MeasurementType } from "@/features/manualMeasurements/types";

interface Props {
  selectedType: MeasurementType | undefined;
  onBack: () => void;
  onAddEntry: () => void;
  onOpenGoal: () => void;
}

function DirectionIcon({ direction }: { direction: string | null }) {
  if (direction === "decrease") return <TrendingDown className="h-3.5 w-3.5 text-primary-text" />;
  if (direction === "increase") return <TrendingUp className="h-3.5 w-3.5 text-success-text" />;
  return <Minus className="h-3.5 w-3.5 text-warning-text" />;
}

export function DetailHeader({ selectedType, onBack, onAddEntry, onOpenGoal }: Props) {
  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-3">
        <Button
          variant="ghost"
          size="icon"
          onClick={onBack}
          aria-label="Retour à la liste des mensurations"
        >
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div>
          <h2 className="text-lg font-semibold">{selectedType?.label || "Mensuration"}</h2>
          {selectedType?.target_value != null && (
            <div className="flex items-center gap-1.5 mt-0.5">
              <DirectionIcon direction={selectedType.target_direction} />
              <span className="text-sm text-muted-foreground">
                {DIRECTION_LABELS[selectedType.target_direction || ""] || ""} vers{" "}
                {selectedType.target_value} {selectedType.unit}
              </span>
            </div>
          )}
        </div>
      </div>
      <div className="flex gap-2">
        <Button variant="outline" size="sm" onClick={onOpenGoal}>
          <Target className="mr-2 h-4 w-4" />
          Objectif
        </Button>
        <Button size="sm" onClick={onAddEntry}>
          <Plus className="mr-2 h-4 w-4" />
          Mesure
        </Button>
      </div>
    </div>
  );
}
