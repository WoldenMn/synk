import { TriangleAlert, Eye } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import type { HealthAlert, AlertEvent } from "@/features/alerts/types";

interface AlertEventsProps {
  alerts: HealthAlert[];
  events: AlertEvent[];
  onMarkSeen: () => void;
}

export function AlertEvents({ alerts, events, onMarkSeen }: AlertEventsProps) {
  const unseenCount = events.filter((e) => !e.seen).length;

  function getAlertLabel(alertId: number) {
    return alerts.find((a) => a.id === alertId)?.label ?? `Alerte #${alertId}`;
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-base">
            <TriangleAlert className="h-4 w-4" />
            Événements récents
            {unseenCount > 0 && (
              <span className="ml-2 rounded-full bg-destructive px-2 py-0.5 text-xs text-destructive-foreground">
                {unseenCount}
              </span>
            )}
          </CardTitle>
          {unseenCount > 0 && (
            <Button variant="outline" size="sm" onClick={onMarkSeen}>
              <Eye className="mr-2 h-4 w-4" />
              Tout marquer lu
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent>
        {events.length === 0 ? (
          <p className="text-sm text-muted-foreground text-center py-4">Aucun événement</p>
        ) : (
          <div className="space-y-2">
            {events.map((evt) => (
              <div
                key={evt.id}
                className={`flex items-center justify-between rounded-md border p-3 ${
                  !evt.seen ? "bg-destructive/5 border-destructive/20" : ""
                }`}
              >
                <div>
                  <p className="text-sm font-medium">{getAlertLabel(evt.alert_id)}</p>
                  <p className="text-xs text-muted-foreground">
                    Valeur : {evt.value}, le {evt.triggered_at.substring(0, 16).replace("T", " ")}
                  </p>
                </div>
                {!evt.seen && <span className="h-2 w-2 rounded-full bg-destructive shrink-0" />}
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
