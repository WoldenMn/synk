import { useId, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/shared/EmptyState";
import { TriangleAlert, Trash, ChevronDown, ChevronRight } from "lucide-react";
import { useErrorLog, usePurgeErrorLog } from "../hooks";
import type { ErrorLogEntry } from "../types";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { formatDateTime } from "@/lib/formatters";
import { Select } from "@/components/ui/select";

const CODES = [
  "AUTH",
  "NETWORK",
  "DATABASE",
  "RATE_LIMIT",
  "TIMEOUT",
  "VALIDATION",
  "CRYPTO",
  "INTERNAL",
  "UNKNOWN",
];

const CODE_TONE: Record<string, string> = {
  AUTH: "bg-warning/15 text-warning-text",
  NETWORK: "bg-primary/15 text-primary-text",
  DATABASE: "bg-destructive/15 text-destructive-text",
  RATE_LIMIT: "bg-warning/15 text-warning-text",
  TIMEOUT: "bg-primary/15 text-primary-text",
};

interface ErrorEntryProps {
  entry: ErrorLogEntry;
  expanded: boolean;
  onToggle: () => void;
}

function ErrorEntry({ entry: e, expanded, onToggle }: ErrorEntryProps) {
  const panelId = `error-panel-${useId()}`;
  return (
    <div className="rounded-md border text-sm">
      <button
        type="button"
        aria-expanded={expanded}
        aria-controls={panelId}
        onClick={onToggle}
        className="flex w-full items-center gap-2 p-2 text-left hover:bg-accent/15"
      >
        {expanded ? (
          <ChevronDown className="h-3.5 w-3.5 shrink-0" />
        ) : (
          <ChevronRight className="h-3.5 w-3.5 shrink-0" />
        )}
        <span
          className={`rounded px-1.5 py-0.5 text-xs font-semibold ${CODE_TONE[e.code] ?? "bg-muted text-muted-foreground"}`}
        >
          {e.code}
        </span>
        <span className="text-xs text-muted-foreground">{e.source}</span>
        <span className="truncate flex-1">{e.message}</span>
        <span className="text-xs text-muted-foreground shrink-0">
          {formatDateTime(e.occurred_at)}
        </span>
      </button>
      {expanded && (
        <div id={panelId} className="border-t px-3 py-2 text-xs text-muted-foreground">
          <p>{e.message}</p>
          {e.context && <p className="mt-1">Contexte : {e.context}</p>}
        </div>
      )}
    </div>
  );
}

export function ErrorLogSection() {
  const [filter, setFilter] = useState<string | null>(null);
  const [expanded, setExpanded] = useState<number | null>(null);
  const { data: entries, isLoading } = useErrorLog(filter);
  const purge = usePurgeErrorLog();
  const confirm = useConfirm();

  async function handlePurge() {
    if (!(await confirm({ title: "Vider le journal d'erreurs ?", destructive: true }))) return;
    purge.mutate();
  }

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0">
        <CardTitle className="flex items-center gap-2 text-base">
          <TriangleAlert className="h-4 w-4" /> Journal d'erreurs
        </CardTitle>
        <div className="flex items-center gap-2">
          <Select
            aria-label="Filtrer par code d'erreur"
            value={filter ?? ""}
            onChange={(e) => setFilter(e.target.value || null)}
            className="h-8 rounded-md border bg-background px-2 text-xs"
          >
            <option value="">Tous les codes</option>
            {CODES.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </Select>
          <Button variant="ghost" size="sm" onClick={handlePurge} disabled={purge.isPending}>
            <Trash className="mr-1 h-3.5 w-3.5" /> Purger
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <p className="text-sm text-muted-foreground">Chargement…</p>
        ) : !entries || entries.length === 0 ? (
          <EmptyState
            icon={TriangleAlert}
            title="Aucune erreur"
            description="Rien à signaler, tout roule."
          />
        ) : (
          <div className="space-y-1">
            {entries.map((e) => (
              <ErrorEntry
                key={e.id}
                entry={e}
                expanded={expanded === e.id}
                onToggle={() => setExpanded(expanded === e.id ? null : e.id)}
              />
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
