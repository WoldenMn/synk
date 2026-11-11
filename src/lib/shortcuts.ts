export type ShortcutDef = {
  key: string;
  ctrl: boolean;
  shift: boolean;
  label: string;
  keyshortcuts: string;
};

export const SHORTCUTS = {
  sync: { key: "s", ctrl: true, shift: true, label: "Ctrl+Maj+S", keyshortcuts: "Control+Shift+S" },
  quickAdd: {
    key: "a",
    ctrl: true,
    shift: true,
    label: "Ctrl+Maj+A",
    keyshortcuts: "Control+Shift+A",
  },
  sidebar: { key: "b", ctrl: true, shift: false, label: "Ctrl+B", keyshortcuts: "Control+B" },
} as const satisfies Record<string, ShortcutDef>;
