import { openUrl } from "@tauri-apps/plugin-opener";
import { CircleAlert, LoaderCircle, RotateCcw, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { CredentialsForm } from "@/features/sources/components/CredentialsForm";
import type { SourceInfo } from "@/features/sources/types";
import { useOAuthFlow, type OAuthStatus } from "@/features/sources/useOAuthFlow";

function CredentialsNotice({ source }: { source: SourceInfo }) {
  if (source.credentials_configured) return null;
  return (
    <div className="space-y-3 rounded-md border p-4 text-left">
      <p className="text-sm text-muted-foreground">
        Synk se connecte avec votre propre application {source.label}.
        {source.developer_url
          ? ` Créez-la gratuitement sur ${new URL(source.developer_url).hostname}, puis collez ici son Client ID et son Client Secret.`
          : " Collez ici son Client ID et son Client Secret."}
      </p>
      {source.developer_url && (
        <Button
          variant="link"
          className="h-auto p-0"
          onClick={() => openUrl(source.developer_url as string)}
        >
          Ouvrir {new URL(source.developer_url).hostname}
        </Button>
      )}
      <CredentialsForm source={source.id} idPrefix={`onboarding-${source.id}`} />
    </div>
  );
}

function ConnectAction({
  status,
  error,
  label,
  credsConfigured,
  onConnect,
  onCancel,
}: {
  status: OAuthStatus;
  error: string | null;
  label: string;
  credsConfigured: boolean;
  onConnect: () => void;
  onCancel: () => void;
}) {
  if (status === "waiting") {
    return (
      <div className="space-y-3">
        <div role="status" className="flex flex-col items-center gap-3 py-4">
          <LoaderCircle className="h-8 w-8 animate-spin text-primary-text" />
          <p className="text-sm text-muted-foreground">En attente de l'autorisation {label}...</p>
          <p className="text-xs text-muted-foreground">
            Autorisez l'accès dans votre navigateur puis revenez ici.
          </p>
        </div>
        <Button variant="outline" className="w-full" onClick={onCancel}>
          <X className="mr-2 h-4 w-4" />
          Annuler la connexion
        </Button>
      </div>
    );
  }
  if (status === "timeout" || status === "error") {
    return (
      <div className="space-y-3">
        <div
          role="alert"
          className="flex items-start gap-2 rounded-md border border-destructive/30 bg-destructive/5 p-3 text-left"
        >
          <CircleAlert className="h-4 w-4 text-destructive-text mt-0.5 shrink-0" />
          <p className="text-xs text-destructive-text">{error ?? "Une erreur est survenue."}</p>
        </div>
        <Button className="w-full" size="lg" onClick={onConnect}>
          <RotateCcw className="mr-2 h-4 w-4" />
          Réessayer
        </Button>
      </div>
    );
  }
  return (
    <Button
      className="w-full"
      size="lg"
      onClick={onConnect}
      disabled={status === "connecting" || !credsConfigured}
    >
      {status === "connecting" ? "Redirection..." : "Connecter mon compte"}
    </Button>
  );
}

export function SourceConnect({
  source,
  onConnected,
  onBack,
}: {
  source: SourceInfo;
  onConnected: () => void;
  onBack: () => void;
}) {
  const { oauthStatus, oauthError, handleConnect, handleCancelConnect } = useOAuthFlow(
    source,
    onConnected
  );

  return (
    <div className="space-y-4">
      <p className="text-sm text-muted-foreground">
        Liez votre compte {source.label} pour synchroniser automatiquement vos données.
      </p>
      <CredentialsNotice source={source} />
      <ConnectAction
        status={oauthStatus}
        error={oauthError}
        label={source.label}
        credsConfigured={source.credentials_configured}
        onConnect={handleConnect}
        onCancel={handleCancelConnect}
      />
      <Button variant="ghost" className="w-full" onClick={onBack}>
        Retour aux sources
      </Button>
    </div>
  );
}
