import { useEffect } from "react";

export type Shortcut = { key: string; ctrl: boolean; shift?: boolean; run: () => void };

function isTyping(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName))
  );
}

function modalIsOpen(): boolean {
  return document.querySelector('[role="dialog"][aria-modal="true"]') !== null;
}

export function useKeyboardShortcuts(shortcuts: Shortcut[]): void {
  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (isTyping(e.target) || e.altKey || e.metaKey || modalIsOpen()) return;
      const hit = shortcuts.find(
        (s) =>
          s.key === e.key.toLowerCase() && s.ctrl === e.ctrlKey && Boolean(s.shift) === e.shiftKey
      );
      if (!hit) return;
      e.preventDefault();
      hit.run();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [shortcuts]);
}
