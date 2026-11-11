import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Unplug } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { sourceDisconnect } from "@/features/sources/api";
import { SOURCES_KEY } from "@/features/sources/hooks";
import { useOAuthFlow } from "@/features/sources/useOAuthFlow";
import { domainLabel } from "@/features/sources/labels";
import { reauthText } from "@/features/sources/reauth";
import type { SourceInfo } from "@/features/sources/types";
import { formatDate } from "@/lib/formatters";
import { reportError } from "@/lib/reportError";
import { ConnectedActions, DisconnectedActions } from "./SourceActions";
import { SourceCredentials } from "./SourceCredentials";

const noop = () => undefined;

function lastSyncText(iso: string | null): string {
  return iso ? `Dernière synchro : ${formatDate(iso)}` : "Jamais synchronisée";
}

export function SourceCard({ source }: { source: SourceInfo }) {
  const queryClient = useQueryClient();
  const confirm = useConfirm();
  const { oauthStatus, oauthError, handleConnect, handleCancelConnect } = useOAuthFlow(
    source,
    noop
  );

  async function handleDisconnect() {
    const confirmed = await confirm({
      title: `Déconnecter ${source.label} ?`,
      description: `Les jetons et les repères de synchronisation de ${source.label} seront effacés : une reconnexion refera la synchronisation initiale. Vos mesures déjà importées restent dans Synk. Synk demandera aussi à ${source.label} de révoquer son accès.`,
      confirmLabel: "Déconnecter",
    });
    if (!confirmed) return;
    try {
      const report = await sourceDisconnect(source.id);
      if (report.warning) {
        toast.warning(`${source.label} est déconnecté de Synk. ${report.warning}`, {
          duration: 12_000,
        });
      }
    } catch (e) {
      reportError(e, { source: "manual", context: `source_disconnect:${source.id}` });
      return;
    }
    handleCancelConnect();
    queryClient.invalidateQueries({ queryKey: SOURCES_KEY });
  }

  const connected = source.connected;

  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3" className="flex items-center gap-2">
          <Unplug className="h-5 w-5" aria-hidden="true" />
          {source.label}
        </CardTitle>
        <CardDescription>
          {source.domains.map(domainLabel).join(", ")}
          {source.domains.length > 0 && ". "}
          {lastSyncText(source.last_sync_at)}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <Badge
            variant={connected ? "default" : "outline"}
            className={
              connected
                ? "bg-success/10 text-success-text border-success/20"
                : "text-muted-foreground"
            }
          >
            {connected ? "Connecté" : "Non connecté"}
          </Badge>
          {connected ? (
            <ConnectedActions source={source} onDisconnect={handleDisconnect} />
          ) : (
            <DisconnectedActions
              source={source}
              status={oauthStatus}
              onConnect={handleConnect}
              onCancel={handleCancelConnect}
            />
          )}
        </div>
        {!connected && oauthError && (
          <p role="alert" className="text-sm text-destructive-text">
            {oauthError}
          </p>
        )}
        {connected && source.reauth && (
          <p role="status" className="text-sm text-warning-text">
            {reauthText(source.label, source.reauth)}
          </p>
        )}
        {source.kind === "oauth" && <SourceCredentials source={source} />}
      </CardContent>
    </Card>
  );
}
