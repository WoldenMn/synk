import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Smartphone } from "lucide-react";
import { useDevices } from "@/features/devices/hooks";
import { SourceBadge } from "@/features/sources/components/SourceBadge";

interface DevicesCardProps {
  visible: boolean;
}

export function DevicesCard({ visible }: DevicesCardProps) {
  const { data: devices } = useDevices();

  if (!visible || !devices || devices.length === 0) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3" className="flex items-center gap-2">
          <Smartphone className="h-5 w-5" />
          Appareils Withings
        </CardTitle>
        <CardDescription>Appareils connectés à votre compte</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {devices.map((device) => (
            <div
              key={device.device_id}
              className="flex items-center justify-between rounded-lg border p-3"
            >
              <div className="space-y-0.5">
                <div className="flex items-center gap-2 text-sm font-medium">
                  {device.model || "Appareil inconnu"}
                  <SourceBadge source="withings" />
                </div>
                <p className="text-xs text-muted-foreground">{device.device_type || ""}</p>
              </div>
              <div className="flex items-center gap-3">
                {device.battery && (
                  <Badge
                    variant="outline"
                    className={
                      device.battery === "high"
                        ? "bg-success/10 text-success-text border-success/20"
                        : device.battery === "medium"
                          ? "bg-warning/10 text-warning-text border-warning/20"
                          : "bg-destructive/10 text-destructive-text border-destructive/20"
                    }
                  >
                    {device.battery === "high"
                      ? "Batterie OK"
                      : device.battery === "medium"
                        ? "Batterie moyenne"
                        : "Batterie faible"}
                  </Badge>
                )}
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
