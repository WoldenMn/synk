import { useState } from "react";
import { logError } from "@/lib/logger";
import { errorText } from "@/lib/errors";
import { revealDataFolder } from "@/lib/reveal";
import type { BootSnapshot } from "@/hooks/useAppReady";
import { getAppDataDir, getBootState } from "@/hooks/appReady/api";

interface BootStuckScreenProps {
  lastSnapshot: BootSnapshot | null;
  lastError: string | null;
}

export function BootStuckScreen({ lastSnapshot, lastError }: BootStuckScreenProps) {
  const [details, setDetails] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [openError, setOpenError] = useState<string | null>(null);
  const [refreshResult, setRefreshResult] = useState<BootSnapshot | string | null>(null);

  const reload = () => window.location.reload();

  const recheck = async () => {
    setRefreshing(true);
    try {
      const snap = await getBootState();
      setRefreshResult(snap);
      if (snap.status === "ready" || snap.status === "failed") {
        setTimeout(reload, 600);
      }
    } catch (e) {
      setRefreshResult(errorText(e));
    } finally {
      setRefreshing(false);
    }
  };

  const openLogsFolder = async () => {
    try {
      const dataDir = await getAppDataDir();
      setOpenError(null);
      await revealDataFolder(dataDir);
    } catch (e) {
      logError("openLogsFolder failed:", e);
      setOpenError(`Ouverture du dossier impossible : ${errorText(e)}`);
    }
  };

  const status = lastSnapshot?.status ?? null;

  return (
    <div className="flex h-screen items-center justify-center bg-background text-foreground p-8 overflow-y-auto">
      <div className="max-w-xl w-full space-y-4">
        <div className="text-center space-y-2">
          <div className="text-4xl text-warning-text" aria-hidden="true">
            !
          </div>
          <h1 className="text-xl font-semibold">Synk ne termine pas son démarrage</h1>
          <p className="text-sm text-muted-foreground">
            Le démarrage dure plus longtemps que d'habitude (plus de 12 secondes). Vous pouvez
            vérifier à nouveau ou recharger l'application ; si rien ne change, ouvrez le dossier de
            données et transmettez le fichier Synk.log au support.
          </p>
        </div>

        <div className="rounded border border-border bg-muted/30 p-4 text-sm" role="status">
          {status === "ready" && (
            <p className="text-success-text">
              Le moteur de Synk est prêt, mais l'écran ne l'a pas su. Cliquez sur « Re-vérifier ».
            </p>
          )}
          {status === "failed" && (
            <p className="text-destructive-text">
              Le démarrage a échoué. Les détails pour le support sont repliés ci-dessous.
            </p>
          )}
          {status === "loading" && (
            <p className="text-warning-text">
              Le démarrage est encore en cours. Patientez un peu, ou ouvrez le dossier de données
              pour lire le journal.
            </p>
          )}
          {status === null && (
            <p className="text-destructive-text">
              Impossible de joindre le moteur de Synk. Rechargez l'application.
            </p>
          )}
        </div>

        <div className="flex flex-col gap-2 items-center">
          <button
            onClick={recheck}
            disabled={refreshing}
            className="w-64 rounded bg-primary px-4 py-2 text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            {refreshing ? "Vérification..." : "Re-vérifier l'état du démarrage"}
          </button>
          <button
            onClick={reload}
            className="w-64 rounded border border-border px-4 py-2 hover:bg-accent/15"
          >
            Recharger l'application
          </button>
          <button
            onClick={openLogsFolder}
            className="w-64 rounded border border-border px-4 py-2 hover:bg-accent/15"
          >
            Ouvrir le dossier de données
          </button>
        </div>

        {openError && (
          <p role="alert" className="text-center text-sm text-destructive-text">
            {openError}
          </p>
        )}

        <details
          open={details}
          onToggle={(e) => setDetails((e.target as HTMLDetailsElement).open)}
          className="text-xs"
        >
          <summary className="text-muted-foreground cursor-pointer">
            Détails pour le support
          </summary>
          <div className="mt-2 space-y-2">
            <div>
              <div className="font-semibold">État du démarrage :</div>
              <pre className="mt-1 overflow-x-auto rounded bg-muted p-2">
                {JSON.stringify(lastSnapshot, null, 2)}
              </pre>
            </div>
            {lastError && (
              <div>
                <div className="font-semibold">Dernière erreur :</div>
                <pre className="mt-1 overflow-x-auto rounded bg-muted p-2">{lastError}</pre>
              </div>
            )}
            {refreshResult && (
              <div>
                <div className="font-semibold">Résultat de la re-vérification :</div>
                <pre className="mt-1 overflow-x-auto rounded bg-muted p-2">
                  {JSON.stringify(refreshResult, null, 2)}
                </pre>
              </div>
            )}
            <div>
              <div className="font-semibold">Journal de Synk :</div>
              <code className="block bg-muted p-2 rounded mt-1 text-xs">
                %LOCALAPPDATA%\com.synk.health\logs\Synk.log
              </code>
            </div>
            <div>
              <div className="font-semibold">Base de données :</div>
              <code className="block bg-muted p-2 rounded mt-1 text-xs">
                %APPDATA%\com.synk.health\synk.db
              </code>
            </div>
          </div>
        </details>
      </div>
    </div>
  );
}
