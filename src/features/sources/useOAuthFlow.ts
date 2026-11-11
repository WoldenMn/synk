import { useState, useEffect, useCallback, useMemo } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { toast } from "sonner";
import { useQueryClient } from "@tanstack/react-query";
import { sourceStartAuth } from "@/features/sources/api";
import { SOURCES_KEY, useSourceEvents } from "@/features/sources/hooks";
import { reportError } from "@/lib/reportError";

const OAUTH_TIMEOUT_MS = 120_000;

export type OAuthStatus = "idle" | "connecting" | "waiting" | "timeout" | "error";

export function useOAuthFlow(source: { id: string; label: string }, onConnected: () => void) {
  const [oauthStatus, setOauthStatus] = useState<OAuthStatus>("idle");
  const [oauthError, setOauthError] = useState<string | null>(null);
  const queryClient = useQueryClient();
  const ids = useMemo(() => [source.id], [source.id]);

  const onSuccess = useCallback(() => {
    toast.success(`Connexion ${source.label} réussie !`);
    setOauthStatus("idle");
    setOauthError(null);
    queryClient.invalidateQueries({ queryKey: SOURCES_KEY });
    onConnected();
  }, [source.label, queryClient, onConnected]);

  const onError = useCallback(
    (_id: string, payload: unknown) => {
      setOauthStatus("error");
      setOauthError(
        reportError(payload, { source: "manual", context: `${source.id}_auth` }).userMessage
      );
    },
    [source.id]
  );

  const onTimeout = useCallback(() => {
    setOauthStatus("timeout");
    setOauthError(
      `Délai d'autorisation ${source.label} dépassé (120s). Vérifiez que la fenêtre du navigateur s'est ouverte et autorisez l'accès.`
    );
    toast.warning(`Délai d'autorisation ${source.label} dépassé.`);
  }, [source.label]);

  useSourceEvents("auth-success", ids, onSuccess);
  useSourceEvents("auth-error", ids, onError);
  useSourceEvents("auth-timeout", ids, onTimeout);

  useEffect(() => {
    if (oauthStatus !== "waiting") return;
    const tid = window.setTimeout(() => {
      setOauthStatus("timeout");
      setOauthError(`Délai d'autorisation ${source.label} dépassé. Réessayez.`);
    }, OAUTH_TIMEOUT_MS + 5000);
    return () => window.clearTimeout(tid);
  }, [oauthStatus, source.label]);

  async function handleConnect() {
    setOauthStatus("connecting");
    setOauthError(null);
    try {
      const url = await sourceStartAuth(source.id);
      await openUrl(url);
      setOauthStatus("waiting");
      toast.success(`Redirection vers ${source.label} : autorisez l'accès puis revenez ici`);
    } catch (err) {
      setOauthStatus("error");
      setOauthError(
        reportError(err, { source: "manual", context: `source_start_auth:${source.id}` })
          .userMessage
      );
    }
  }

  function handleCancelConnect() {
    setOauthStatus("idle");
    setOauthError(null);
  }

  return { oauthStatus, oauthError, handleConnect, handleCancelConnect };
}
