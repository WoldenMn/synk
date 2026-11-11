import { KeyRound } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { CredentialsForm } from "@/features/sources/components/CredentialsForm";
import { sourceHelp } from "@/features/sources/help";
import type { SourceInfo } from "@/features/sources/types";

export function SourceCredentials({ source }: { source: SourceInfo }) {
  return (
    <details open={!source.credentials_configured} className="rounded-lg border p-3">
      <summary className="flex cursor-pointer items-center gap-2 text-sm font-medium">
        <KeyRound className="h-4 w-4" aria-hidden="true" />
        Identifiants de l'application {source.label}
        {source.credentials_configured && <Badge variant="secondary">Enregistrés</Badge>}
      </summary>
      <div className="mt-3 space-y-3">
        <p className="text-sm text-muted-foreground">
          Créez votre propre application
          {source.developer_url && (
            <>
              {" "}
              sur{" "}
              <a
                href={source.developer_url}
                target="_blank"
                rel="noopener noreferrer"
                className="underline"
              >
                {new URL(source.developer_url).hostname}
              </a>
            </>
          )}{" "}
          et saisissez les identifiants ici.
        </p>
        {sourceHelp(source.id).length > 0 && (
          <ul className="list-disc space-y-1 pl-5 text-sm text-muted-foreground">
            {sourceHelp(source.id).map((line) => (
              <li key={line}>{line}</li>
            ))}
          </ul>
        )}
        {source.redirect_uri && (
          <p className="text-sm text-muted-foreground">
            URL de rappel à déclarer :{" "}
            <code className="rounded bg-muted px-1 py-0.5 text-xs">{source.redirect_uri}</code>
          </p>
        )}
        <CredentialsForm source={source.id} idPrefix={`settings-${source.id}`} />
      </div>
    </details>
  );
}
