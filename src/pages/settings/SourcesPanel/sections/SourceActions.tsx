import { ExternalLink, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useSync } from "@/hooks/useSync";
import type { OAuthStatus } from "@/features/sources/useOAuthFlow";
import type { SourceInfo } from "@/features/sources/types";

function Named({ label, children }: { label: string; children: string }) {
  return (
    <>
      {children} <span className="sr-only">{label}</span>
    </>
  );
}

export function ConnectedActions({
  source,
  onDisconnect,
}: {
  source: SourceInfo;
  onDisconnect: () => void;
}) {
  const { sync, isSyncing, syncingHere } = useSync(source.id);
  return (
    <div className="flex gap-2">
      <Button variant="outline" size="sm" onClick={sync} disabled={isSyncing}>
        <RefreshCw className={`mr-2 h-4 w-4 ${syncingHere ? "animate-spin" : ""}`} />
        <Named label={source.label}>{syncingHere ? "Synchronisation..." : "Synchroniser"}</Named>
      </Button>
      <Button variant="ghost" size="sm" onClick={onDisconnect}>
        <Named label={source.label}>Déconnecter</Named>
      </Button>
    </div>
  );
}

export function DisconnectedActions({
  source,
  status,
  onConnect,
  onCancel,
}: {
  source: SourceInfo;
  status: OAuthStatus;
  onConnect: () => void;
  onCancel: () => void;
}) {
  const connecting = status === "connecting" || status === "waiting";
  const caption = !source.credentials_configured
    ? "Saisissez d'abord les identifiants"
    : connecting
      ? "En attente..."
      : "Connecter";
  return (
    <div className="flex gap-2">
      <Button
        variant="outline"
        onClick={onConnect}
        disabled={connecting || !source.credentials_configured}
      >
        <ExternalLink className="mr-2 h-4 w-4" />
        <Named label={source.label}>{caption}</Named>
      </Button>
      {status === "waiting" && (
        <Button variant="ghost" onClick={onCancel}>
          <Named label={source.label}>Annuler la connexion</Named>
        </Button>
      )}
    </div>
  );
}
