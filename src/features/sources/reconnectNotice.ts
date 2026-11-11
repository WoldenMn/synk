import type { InvalidatedSecrets, InvalidatedSource } from "./types";

export interface SourceState {
  connected: boolean;
  credentialsConfigured: boolean;
}

export type SourceStates = Record<string, SourceState>;

export function isReconnectRequired(
  secrets: InvalidatedSecrets | null | undefined
): secrets is InvalidatedSecrets {
  return Boolean(secrets && secrets.sources.length > 0);
}

function names(secrets: InvalidatedSecrets): string {
  return secrets.sources.map((s) => s.label).join(", ");
}

export function reconnectTitle(secrets: InvalidatedSecrets): string {
  return `Reconnexion ${names(secrets)} nécessaire`;
}

export function reconnectReason(secrets: InvalidatedSecrets): string {
  return (
    `Synk n'a pas pu déchiffrer les identifiants enregistrés pour ${names(secrets)} : la clé qui ` +
    "les protégeait n'était plus disponible au démarrage. Ils ont été effacés, vos mesures santé " +
    "ne sont pas touchées."
  );
}

function stepsFor(source: InvalidatedSource, state: SourceState | undefined): string[] {
  const steps: string[] = [];
  if (source.credentials && !state?.credentialsConfigured) {
    steps.push(`${source.label} : saisissez à nouveau votre Client ID et votre Client Secret.`);
  }
  if (!state?.connected) {
    steps.push(`${source.label} : cliquez sur Connecter pour autoriser Synk de nouveau.`);
  }
  return steps;
}

export function reconnectSteps(
  secrets: InvalidatedSecrets | null | undefined,
  states: SourceStates
): string[] {
  if (!isReconnectRequired(secrets)) return [];
  return secrets.sources.flatMap((source) => stepsFor(source, states[source.source]));
}
