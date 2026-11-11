// Logger centralise env-aware. En production (build Tauri), seuls les warn et
// error sont emis. En dev (vite dev), tout passe. Prefixe systematique "[Synk]"
// pour faciliter le filtrage devtools.

const isDev = import.meta.env.DEV;

export function logDebug(...args: unknown[]): void {
  if (isDev) {
    // eslint-disable-next-line no-console -- logger central, seul site autorise
    console.log("[Synk]", ...args);
  }
}

export function logInfo(...args: unknown[]): void {
  if (isDev) {
    // eslint-disable-next-line no-console -- logger central, seul site autorise
    console.info("[Synk]", ...args);
  }
}

export function logWarn(...args: unknown[]): void {
  // eslint-disable-next-line no-console -- logger central, seul site autorise
  console.warn("[Synk]", ...args);
}

export function logError(...args: unknown[]): void {
  // eslint-disable-next-line no-console -- logger central, seul site autorise
  console.error("[Synk]", ...args);
}
