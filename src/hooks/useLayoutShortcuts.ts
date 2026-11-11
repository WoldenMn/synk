import { useMemo } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { useUiStore } from "@/stores/uiStore";
import { SHORTCUTS } from "@/lib/shortcuts";
import { showsQuickAdd } from "@/lib/quickAdd";
import { useKeyboardShortcuts } from "./useKeyboardShortcuts";

export function useLayoutShortcuts(sync: () => void): void {
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);
  const setQuickAddOpen = useUiStore((s) => s.setQuickAddOpen);

  const shortcuts = useMemo(
    () => [
      { ...SHORTCUTS.sync, run: sync },
      {
        ...SHORTCUTS.quickAdd,
        run: () => {
          if (!showsQuickAdd(pathname)) navigate("/");
          setQuickAddOpen(true);
        },
      },
      { ...SHORTCUTS.sidebar, run: toggleSidebar },
    ],
    [sync, pathname, navigate, setQuickAddOpen, toggleSidebar]
  );

  useKeyboardShortcuts(shortcuts);
}
