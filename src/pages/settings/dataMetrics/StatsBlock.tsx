import { FileText } from "lucide-react";
import { useDbMetrics } from "@/features/dashboard/hooks";
import { useAuditMetrics } from "@/features/audit/hooks";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} Ko`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} Go`;
}

function formatDate(s: string | null): string {
  if (!s) return "—";
  return s.substring(0, 10);
}

export function StatsBlock() {
  const { data: dbMetrics } = useDbMetrics();
  const { data: auditMetrics } = useAuditMetrics();

  return (
    <>
      <div className="grid grid-cols-2 gap-3 text-sm">
        <div>
          <p className="text-muted-foreground">Taille DB</p>
          <p className="font-medium">{dbMetrics ? formatBytes(dbMetrics.db_size_bytes) : "..."}</p>
        </div>
        <div>
          <p className="text-muted-foreground">Audit log</p>
          <p className="font-medium">
            {auditMetrics ? `${auditMetrics.total_count} entrées` : "..."}
          </p>
          {auditMetrics && (
            <p className="text-xs text-muted-foreground">
              {auditMetrics.insert_count} insertions, {auditMetrics.delete_count} suppressions
            </p>
          )}
          {auditMetrics?.oldest_at && (
            <p className="text-xs text-muted-foreground">
              du {formatDate(auditMetrics.oldest_at)} au {formatDate(auditMetrics.newest_at)}
            </p>
          )}
        </div>
      </div>

      {dbMetrics && (
        <div>
          <p className="text-sm font-medium mb-2 flex items-center gap-1">
            <FileText className="h-4 w-4" />
            Mesures
          </p>
          <div className="grid grid-cols-3 gap-2 text-xs">
            <div>
              <span className="text-muted-foreground">Body</span>{" "}
              <span className="font-medium">{dbMetrics.body_measures_count}</span>
            </div>
            <div>
              <span className="text-muted-foreground">BP</span>{" "}
              <span className="font-medium">{dbMetrics.blood_pressure_count}</span>
            </div>
            <div>
              <span className="text-muted-foreground">Sleep</span>{" "}
              <span className="font-medium">{dbMetrics.sleep_count}</span>
            </div>
            <div>
              <span className="text-muted-foreground">Activity</span>{" "}
              <span className="font-medium">{dbMetrics.activity_count}</span>
            </div>
            <div>
              <span className="text-muted-foreground">Workouts</span>{" "}
              <span className="font-medium">{dbMetrics.workouts_count}</span>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
