type IdleWindow = Window & {
  requestIdleCallback?: (cb: () => void, opts?: { timeout: number }) => number;
  cancelIdleCallback?: (handle: number) => void;
};

/**
 * Lance `cb` quand le navigateur est inactif (au plus tard apres 2 s), ou apres
 * 200 ms si requestIdleCallback n'existe pas. Renvoie la fonction d'annulation.
 */
export function scheduleIdle(cb: () => void): () => void {
  const win = window as IdleWindow;
  if (win.requestIdleCallback && win.cancelIdleCallback) {
    const handle = win.requestIdleCallback(cb, { timeout: 2000 });
    return () => win.cancelIdleCallback?.(handle);
  }
  const handle = window.setTimeout(cb, 200);
  return () => window.clearTimeout(handle);
}
