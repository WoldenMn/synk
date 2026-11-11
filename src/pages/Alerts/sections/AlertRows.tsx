import { Trash } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { HealthAlert } from "@/features/alerts/types";
import { getMetricLabel } from "../constants";

interface AlertRowsProps {
  alerts: HealthAlert[];
  onDelete: (id: number) => void;
}

export function AlertRows({ alerts, onDelete }: AlertRowsProps) {
  return (
    <>
      {alerts.length === 0 ? (
        <p className="text-sm text-muted-foreground text-center py-6">Aucune alerte configurée</p>
      ) : (
        <div className="space-y-2">
          {alerts.map((alert) => (
            <div key={alert.id} className="flex items-center justify-between rounded-md border p-3">
              <div>
                <p className="text-sm font-medium">{alert.label}</p>
                <p className="text-xs text-muted-foreground">
                  {getMetricLabel(alert.metric_key)} {alert.condition === "above" ? ">" : "<"}{" "}
                  {alert.threshold}
                </p>
              </div>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 text-destructive-text"
                onClick={() => onDelete(alert.id)}
                aria-label={`Supprimer l'alerte ${alert.label}`}
              >
                <Trash className="h-4 w-4" />
              </Button>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
