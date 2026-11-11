import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Select } from "@/components/ui/select";
import { Clock } from "lucide-react";
import { useSyncStore } from "@/stores/syncStore";

export function SyncIntervalCard() {
  const syncInterval = useSyncStore((s) => s.syncInterval);
  const setSyncInterval = useSyncStore((s) => s.setSyncInterval);

  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3" className="flex items-center gap-2">
          <Clock className="h-5 w-5" />
          Synchronisation automatique
        </CardTitle>
        <CardDescription>
          Synchronise automatiquement au lancement si le délai est dépassé
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium">Intervalle</p>
            <p className="text-sm text-muted-foreground">
              {syncInterval === 0 ? "Désactivée" : `Toutes les ${syncInterval}h`}
            </p>
          </div>
          <Select
            aria-label="Intervalle de synchronisation automatique"
            className="w-[140px]"
            value={String(syncInterval)}
            onChange={(e: React.ChangeEvent<HTMLSelectElement>) =>
              setSyncInterval(Number(e.target.value))
            }
          >
            <option value="0">Désactivée</option>
            <option value="1">1 heure</option>
            <option value="6">6 heures</option>
            <option value="12">12 heures</option>
            <option value="24">24 heures</option>
          </Select>
        </div>
      </CardContent>
    </Card>
  );
}
