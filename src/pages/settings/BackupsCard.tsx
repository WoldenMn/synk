import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { LoadingSkeleton } from "@/components/shared";
import { Archive } from "lucide-react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useBackups, useCreateBackup } from "@/features/backups/hooks";
import type { BackupInfo } from "@/features/backups/api";
import { formatDateTime, formatNumber } from "@/lib/formatters";
import { formatError } from "@/lib/errors";
import { reportError } from "@/lib/reportError";

const KIND_LABELS: Record<BackupInfo["kind"], string> = {
  auto: "Avant mise à jour",
  quotidienne: "Quotidienne",
  manuelle: "Manuelle",
};

function formatSize(bytes: number): string {
  return bytes >= 1048576
    ? `${formatNumber(bytes / 1048576, 1)} Mo`
    : `${formatNumber(bytes / 1024, 0)} Ko`;
}

export function BackupsCard() {
  const { data: backups, isLoading, isError, error, refetch } = useBackups();
  const create = useCreateBackup();
  const newest = backups?.[0];

  const reveal = async () => {
    if (!newest) return;
    try {
      await revealItemInDir(newest.path);
    } catch (e) {
      reportError(e, { source: "manual", context: "backups_reveal" });
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3" className="flex items-center gap-2">
          <Archive className="h-5 w-5" aria-hidden="true" />
          Sauvegardes
        </CardTitle>
        <CardDescription>
          Synk copie votre base chaque jour au démarrage (14 gardées) et avant chaque mise à jour du
          schéma. Ces copies restent sur ce disque.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {isLoading ? (
          <LoadingSkeleton rows={3} />
        ) : isError ? (
          <div role="alert" className="space-y-2">
            <p className="text-sm">
              Liste des sauvegardes illisible : {formatError(error).message}
            </p>
            <Button size="sm" variant="outline" onClick={() => refetch()}>
              Réessayer
            </Button>
          </div>
        ) : backups && backups.length > 0 ? (
          <ul className="space-y-1 text-sm">
            {backups.slice(0, 5).map((b) => (
              <li key={b.path} className="flex justify-between gap-4">
                <span>
                  {formatDateTime(b.modified)} ({KIND_LABELS[b.kind]})
                </span>
                <span className="text-muted-foreground">{formatSize(b.size_bytes)}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-muted-foreground">Aucune sauvegarde pour l'instant.</p>
        )}
        <div className="flex flex-wrap gap-2">
          <Button onClick={() => create.mutate()} disabled={create.isPending}>
            {create.isPending ? "Sauvegarde…" : "Créer une sauvegarde maintenant"}
          </Button>
          <Button variant="outline" onClick={reveal} disabled={!newest}>
            Afficher dans l'Explorateur
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
