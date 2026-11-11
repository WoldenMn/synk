import { useId } from "react";
import { FlaskConical, ChevronDown, ChevronUp, Trash } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { LoadingSkeleton, EmptyState } from "@/components/shared";
import { formatDate, formatNumber } from "@/lib/formatters";
import type { LabResult } from "@/features/labs/types";
import { isInRange, StatusDot, TrendArrow, MiniLabChart } from "../helpers";

interface Props {
  loading: boolean;
  groupedByTest: Map<string, LabResult[]>;
  expandedTest: string | null;
  setExpandedTest: (v: string | null) => void;
  onDelete: (id: number) => void;
}

interface CardProps {
  testName: string;
  items: LabResult[];
  isExpanded: boolean;
  onToggle: () => void;
  onDelete: (id: number) => void;
}

function LabAnalysisCard({ testName, items, isExpanded, onToggle, onDelete }: CardProps) {
  const panelId = `lab-panel-${useId()}`;
  const latest = items[0];
  if (latest === undefined) return null;
  const inRange = isInRange(latest.value, latest.reference_min, latest.reference_max);

  return (
    <Card className="overflow-hidden">
      <button
        type="button"
        aria-expanded={isExpanded}
        aria-controls={panelId}
        onClick={onToggle}
        className="flex w-full items-center justify-between p-4 text-left hover:bg-accent/15 transition-colors"
      >
        <div className="flex items-center gap-3 min-w-0 flex-1">
          <StatusDot inRange={inRange} />
          <div className="min-w-0">
            <p className="text-sm font-semibold truncate">{testName}</p>
            <p className="text-xs text-muted-foreground">
              {formatDate(latest.date)}
              {latest.unit ? `, ${latest.unit}` : ""}
              {items.length > 1 ? `, ${items.length} résultats` : ""}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-3 shrink-0 ml-2">
          <MiniLabChart items={items} />
          <TrendArrow items={items} />
          <span
            className={`text-base font-bold tabular-nums ${
              inRange === null ? "" : inRange ? "text-success-text" : "text-destructive-text"
            }`}
          >
            {formatNumber(latest.value)}
          </span>
          {latest.reference_min != null || latest.reference_max != null ? (
            <span className="text-xs text-muted-foreground w-16 text-right">
              {latest.reference_min ?? "?"} – {latest.reference_max ?? "?"}
            </span>
          ) : null}
          {isExpanded ? (
            <ChevronUp className="h-4 w-4 text-muted-foreground" />
          ) : (
            <ChevronDown className="h-4 w-4 text-muted-foreground" />
          )}
        </div>
      </button>

      {isExpanded && (
        <div id={panelId} className="border-t bg-muted/20">
          <div className="[&>:not([hidden])~:not([hidden])]:border-t">
            {items.map((r) => {
              const rInRange = isInRange(r.value, r.reference_min, r.reference_max);
              return (
                <div key={r.id} className="flex items-center justify-between px-4 py-2.5 group">
                  <div className="flex items-center gap-3">
                    <StatusDot inRange={rInRange} />
                    <span className="text-sm text-muted-foreground w-24">{formatDate(r.date)}</span>
                    <span
                      className={`text-sm font-medium tabular-nums ${
                        rInRange === null
                          ? ""
                          : rInRange
                            ? "text-success-text"
                            : "text-destructive-text"
                      }`}
                    >
                      {formatNumber(r.value)} {r.unit ?? ""}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    {r.notes && <span className="text-xs text-muted-foreground">{r.notes}</span>}
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus-visible:opacity-100 transition-opacity"
                      onClick={(e) => {
                        e.stopPropagation();
                        onDelete(r.id);
                      }}
                      aria-label="Supprimer le résultat"
                    >
                      <Trash className="h-3.5 w-3.5 text-destructive-text" />
                    </Button>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </Card>
  );
}

export function LabAnalysesList({
  loading,
  groupedByTest,
  expandedTest,
  setExpandedTest,
  onDelete,
}: Props) {
  if (loading) {
    return <LoadingSkeleton rows={4} height="h-16" />;
  }

  if (groupedByTest.size === 0) {
    return (
      <Card>
        <CardContent className="py-8">
          <EmptyState
            icon={FlaskConical}
            title="Aucun résultat de laboratoire"
            description="Ajoutez vos analyses de sang pour les suivre"
          />
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-2">
      {Array.from(groupedByTest.entries()).map(([testName, items]) => (
        <LabAnalysisCard
          key={testName}
          testName={testName}
          items={items}
          isExpanded={expandedTest === testName}
          onToggle={() => setExpandedTest(expandedTest === testName ? null : testName)}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}
