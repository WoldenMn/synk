import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useSetCredentials, useSourceCredentials } from "../hooks";
import { sourceLabel } from "../labels";

interface CredentialsFormProps {
  source: string;
  idPrefix: string;
}

export function CredentialsForm({ source, idPrefix }: CredentialsFormProps) {
  const { data: credsConfigured } = useSourceCredentials(source);
  const setCreds = useSetCredentials(source);
  const [clientId, setClientId] = useState("");
  const [clientSecret, setClientSecret] = useState("");
  const label = sourceLabel(source);

  const handleSave = () => {
    if (!clientId.trim() || !clientSecret.trim()) return;
    setCreds.mutate(
      { clientId: clientId.trim(), clientSecret: clientSecret.trim() },
      {
        onSuccess: () => {
          setClientId("");
          setClientSecret("");
        },
      }
    );
  };

  return (
    <div className="space-y-3">
      <div className="space-y-2">
        <label htmlFor={`${idPrefix}-client-id`} className="text-sm font-medium">
          Client ID
        </label>
        <Input
          id={`${idPrefix}-client-id`}
          value={clientId}
          onChange={(e) => setClientId(e.target.value)}
          placeholder={
            credsConfigured ? "**** (déjà configuré)" : `Collez votre Client ID ${label}`
          }
        />
      </div>
      <div className="space-y-2">
        <label htmlFor={`${idPrefix}-client-secret`} className="text-sm font-medium">
          Client Secret
        </label>
        <Input
          id={`${idPrefix}-client-secret`}
          type="password"
          value={clientSecret}
          onChange={(e) => setClientSecret(e.target.value)}
          placeholder={
            credsConfigured ? "**** (déjà configuré)" : `Collez votre Client Secret ${label}`
          }
        />
      </div>
      <Button
        onClick={handleSave}
        disabled={!clientId.trim() || !clientSecret.trim() || setCreds.isPending}
      >
        {setCreds.isPending ? "Enregistrement…" : "Enregistrer les identifiants"}
      </Button>
    </div>
  );
}
