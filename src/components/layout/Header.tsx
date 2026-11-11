import type { RefObject } from "react";
import { useLocation } from "react-router-dom";
import { Sun, Moon, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { SHORTCUTS } from "@/lib/shortcuts";
import { Badge } from "@/components/ui/badge";
import { useUiStore } from "@/stores/uiStore";
import { useLastSuccessfulSync } from "@/features/syncLog/hooks";
import { useSync } from "@/hooks/useSync";
import { formatDateTime } from "@/lib/formatters";
import { PAGE_TITLES } from "@/lib/constants";

export function Header({ headingRef }: { headingRef: RefObject<HTMLHeadingElement | null> }) {
  const location = useLocation();
  const theme = useUiStore((s) => s.theme);
  const toggleTheme = useUiStore((s) => s.toggleTheme);
  const { data: lastSyncAt, isError: lastSyncFailed } = useLastSuccessfulSync();
  const { sync, isSyncing, syncProgress } = useSync();

  const title = PAGE_TITLES[location.pathname] ?? "";

  return (
    <header className="flex h-14 items-center justify-between border-b px-6">
      <h1 ref={headingRef} tabIndex={-1} className="text-2xl font-semibold">
        {title}
      </h1>

      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2" aria-live="polite" aria-atomic="true">
          {isSyncing ? (
            <Badge variant="secondary" className="gap-1.5">
              <RefreshCw className="h-3 w-3 animate-spin" />
              {syncProgress || "Synchronisation..."}
            </Badge>
          ) : lastSyncAt ? (
            <Badge variant="outline" className="text-muted-foreground font-normal">
              Synchronisé le {formatDateTime(lastSyncAt)}
            </Badge>
          ) : lastSyncFailed ? (
            <Badge variant="outline" className="text-muted-foreground font-normal">
              Dernière synchro inconnue
            </Badge>
          ) : (
            <Badge variant="outline" className="text-muted-foreground font-normal">
              Jamais synchronisé
            </Badge>
          )}
        </div>

        <Tooltip content={`Synchroniser (${SHORTCUTS.sync.label})`} side="bottom">
          <Button
            variant="ghost"
            size="icon"
            onClick={sync}
            disabled={isSyncing}
            aria-label="Synchroniser"
            aria-keyshortcuts={SHORTCUTS.sync.keyshortcuts}
          >
            <RefreshCw className={`h-5 w-5 ${isSyncing ? "animate-spin" : ""}`} />
          </Button>
        </Tooltip>

        <Button
          variant="ghost"
          size="icon"
          onClick={toggleTheme}
          aria-label={theme === "dark" ? "Mode clair" : "Mode sombre"}
        >
          {theme === "dark" ? <Sun className="h-5 w-5" /> : <Moon className="h-5 w-5" />}
        </Button>
      </div>
    </header>
  );
}
