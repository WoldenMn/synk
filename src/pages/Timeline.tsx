import {
  Activity as ActivityIcon,
  RefreshCw,
  TriangleAlert,
  CircleCheck,
  Clock,
  Trash,
} from "lucide-react";
import { ErrorLogSection } from "@/features/errorLog/components/ErrorLogSection";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { LoadingSkeleton, EmptyState } from "@/components/shared";
import { useSyncLog, useClearSyncLog } from "@/features/syncLog/hooks";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { formatDate } from "@/lib/formatters";
import { SourceBadge } from "@/features/sources/components/SourceBadge";

function formatDuration(ms: number | null): string {
  if (ms == null) return "—";
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function StatusIcon({ status }: { status: string }) {
  if (status === "success") {
    return (
      <span className="inline-flex text-success-text">
        <CircleCheck className="h-5 w-5" />
        <span className="sr-only">Réussie</span>
      </span>
    );
  }
  if (status === "error") {
    return (
      <span className="inline-flex text-destructive-text">
        <TriangleAlert className="h-5 w-5" />
        <span className="sr-only">Échec</span>
      </span>
    );
  }
  return (
    <span className="inline-flex text-warning-text">
      <Clock className="h-5 w-5" />
      <span className="sr-only">En cours</span>
    </span>
  );
}

export default function Timeline() {
  const { data: logs = [], isLoading } = useSyncLog(200);
  const clear = useClearSyncLog();
  const confirm = useConfirm();

  const totalRecords = logs.reduce((sum, l) => sum + (l.records_added ?? 0), 0);
  const errorCount = logs.filter((l) => l.status === "error").length;
  const successRate =
    logs.length > 0 ? Math.round(((logs.length - errorCount) / logs.length) * 100) : 100;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between flex-wrap gap-4">
        <div>
          <p className="text-muted-foreground">Historique des synchronisations</p>
        </div>
        {logs.length > 0 && (
          <Button
            variant="outline"
            size="sm"
            onClick={async () => {
              if (
                await confirm({
                  title: "Vider l'historique de sync ?",
                  description:
                    "La dernière synchronisation réussie de chaque type de données est conservée.",
                  destructive: true,
                })
              )
                clear.mutate();
            }}
          >
            <Trash className="mr-2 h-4 w-4" />
            Vider
          </Button>
        )}
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="text-sm text-muted-foreground">Total syncs</div>
            <div className="text-2xl font-bold mt-1">{logs.length}</div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="text-sm text-muted-foreground">Enregistrements</div>
            <div className="text-2xl font-bold mt-1">{totalRecords.toLocaleString()}</div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="text-sm text-muted-foreground">Taux de succès</div>
            <div className="text-2xl font-bold mt-1">{successRate}%</div>
          </CardContent>
        </Card>
      </div>

      {isLoading ? (
        <LoadingSkeleton />
      ) : logs.length === 0 ? (
        <EmptyState
          icon={<ActivityIcon className="h-12 w-12" />}
          title="Aucune sync enregistrée"
          description="L'historique apparaîtra ici après vos prochaines synchronisations."
        />
      ) : (
        <Card>
          <CardHeader>
            <CardTitle className="text-base flex items-center gap-2">
              <RefreshCw className="h-4 w-4" />
              Historique
            </CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            <div className="[&>:not([hidden])~:not([hidden])]:border-t">
              {logs.map((log, idx) => (
                <div
                  key={log.id}
                  className={`flex items-center gap-4 px-6 py-3 hover:bg-muted/30 ${idx < 20 ? "animate-fade-in-up" : ""}`}
                  style={idx < 20 ? { animationDelay: `${idx * 10}ms` } : undefined}
                >
                  <StatusIcon status={log.status} />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium capitalize">{log.data_type}</span>
                      <SourceBadge source={log.source} />
                      {log.records_added != null && log.records_added > 0 && (
                        <span className="text-xs bg-primary/10 text-primary-text px-2 py-0.5 rounded-full">
                          +{log.records_added}
                        </span>
                      )}
                    </div>
                    {log.error && (
                      <div className="text-xs text-destructive-text truncate mt-0.5">
                        {log.error}
                      </div>
                    )}
                  </div>
                  <div className="text-right text-xs text-muted-foreground">
                    <div>{formatDate(log.started_at)}</div>
                    <div>{formatDuration(log.duration_ms)}</div>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      <ErrorLogSection />
    </div>
  );
}
