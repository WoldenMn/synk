import type { ElementType, ReactNode } from "react";
import { RotateCcwClock } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { TimeRangeValue } from "@/lib/timeRange";
import { EmptyState } from "./EmptyState";

interface PeriodEmptyStateProps {
  icon: ElementType | ReactNode;
  /** Titre quand la periode choisie est vide, ex. « Aucune pesée sur cette période ». */
  periodTitle: string;
  /** Titre quand tout l'historique est vide, ex. « Aucune pesée enregistrée ». */
  noneTitle: string;
  /** Comment remplir un historique vide : synchronisation Withings, saisie manuelle. */
  noneDescription: string;
  timeRange: TimeRangeValue;
  onShowAll: () => void;
  compact?: boolean;
}

// Une periode vide ne veut pas dire un historique vide : sans synchronisation
// recente, les 30 derniers jours peuvent l'etre alors que des mois de mesures
// existent. Ce composant le dit et propose d'elargir, au lieu d'inviter a une
// « premiere » saisie.
export function PeriodEmptyState({
  icon,
  periodTitle,
  noneTitle,
  noneDescription,
  timeRange,
  onShowAll,
  compact,
}: PeriodEmptyStateProps) {
  if (timeRange === "all") {
    return (
      <EmptyState icon={icon} title={noneTitle} description={noneDescription} compact={compact} />
    );
  }
  return (
    <EmptyState
      icon={icon}
      title={periodTitle}
      compact={compact}
      description="Élargissez la période pour retrouver votre historique plus ancien."
      action={
        <Button variant="outline" size="sm" onClick={onShowAll}>
          <RotateCcwClock className="mr-2 h-4 w-4" />
          Afficher tout l'historique
        </Button>
      }
    />
  );
}
