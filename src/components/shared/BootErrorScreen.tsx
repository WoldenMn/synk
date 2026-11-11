import { useState } from "react";
import { TriangleAlert, CircleX } from "lucide-react";
import { Button } from "@/components/ui/button";
import { logError } from "@/lib/logger";
import { errorText } from "@/lib/errors";
import { revealDataFolder } from "@/lib/reveal";
import type { BootErrorPayload } from "@/hooks/useAppReady";
import { getAppDataDir, restoreLatestBackup, retryBoot } from "@/hooks/appReady/api";
import { bootErrorCopy } from "./bootErrorCopy";

interface BootErrorScreenProps {
  error: BootErrorPayload;
}

export function BootErrorScreen({ error }: BootErrorScreenProps) {
  const [details, setDetails] = useState(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const copy = bootErrorCopy(error);

  // Le demarrage tourne cote Rust : recharger la page seule rejouerait le meme
  // echec. retry_boot relance le boot, puis la page repart de l'etat de chargement.
  const retry = async () => {
    setBusy(true);
    try {
      await retryBoot();
      window.location.reload();
    } catch (e) {
      logError("retry_boot failed:", e);
      setMessage(`Nouvelle tentative impossible : ${errorText(e)}`);
      setBusy(false);
    }
  };

  const openFolder = async () => {
    try {
      const dir = await getAppDataDir();
      await revealDataFolder(dir);
    } catch (e) {
      logError("openFolder failed:", e);
      setMessage(`Ouverture du dossier impossible : ${errorText(e)}`);
    }
  };

  const restore = async () => {
    setBusy(true);
    try {
      const report = await restoreLatestBackup();
      const skipped = report.skipped?.length
        ? ` ${report.skipped.length} sauvegarde(s) illisible(s) écartée(s).`
        : "";
      setMessage(`Sauvegarde restaurée.${skipped} Redémarrage…`);
      await retry();
    } catch (e) {
      logError("restore_latest_backup failed:", e);
      setMessage(`Échec de la restauration : ${errorText(e)}`);
      setBusy(false);
    }
  };

  const Icon = copy.critical ? CircleX : TriangleAlert;
  const iconColor = copy.critical ? "text-destructive-text" : "text-warning-text";

  return (
    <div className="flex h-screen items-center justify-center bg-background p-8 text-foreground">
      <div className="w-full max-w-lg space-y-4 text-center">
        <div className="flex justify-center" aria-hidden="true">
          <Icon className={`h-12 w-12 ${iconColor}`} />
        </div>
        <h1 className="text-xl font-semibold">{copy.title}</h1>
        <p className="text-sm text-muted-foreground">{copy.description}</p>

        <div className="flex flex-col items-center gap-2 pt-2">
          {copy.canRestore && (
            <Button onClick={restore} disabled={busy} className="w-64">
              {busy ? "Restauration…" : "Restaurer la sauvegarde"}
            </Button>
          )}
          <Button
            onClick={retry}
            disabled={busy}
            variant={copy.canRestore ? "outline" : "default"}
            className="w-64"
          >
            Réessayer
          </Button>
          {copy.canOpenFolder && (
            <Button onClick={openFolder} variant="outline" className="w-64">
              Ouvrir le dossier des données
            </Button>
          )}
          {copy.canQuit && (
            <Button onClick={() => window.close()} variant="outline" className="w-64">
              Quitter
            </Button>
          )}
        </div>

        {message && (
          <p role="status" className="pt-2 text-xs text-muted-foreground">
            {message}
          </p>
        )}

        <details
          open={details}
          onToggle={(e) => setDetails((e.target as HTMLDetailsElement).open)}
          className="pt-4 text-left"
        >
          <summary className="cursor-pointer text-xs text-muted-foreground">
            Voir les détails techniques
          </summary>
          <pre className="mt-2 overflow-x-auto rounded bg-muted p-3 text-xs">
            {JSON.stringify(error, null, 2)}
          </pre>
        </details>
      </div>
    </div>
  );
}
