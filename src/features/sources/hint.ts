export function historyHint(connected: boolean | undefined, complement: string): string {
  const action = connected
    ? "Lancez une synchronisation depuis l'en-tête"
    : "Connectez une source dans Paramètres";
  return `${action} ${complement}.`;
}
