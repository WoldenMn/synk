import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { HeartPulse, Info } from "lucide-react";
import { TimeRangeSelector } from "@/components/charts/TimeRangeSelector";
import { PeriodEmptyState } from "@/components/shared/PeriodEmptyState";
import { DataTable } from "@/components/shared/DataTable";
import type { HeartRecord } from "@/features/heart/types";
import { sourceColumn } from "@/features/sources/columns";
import { useHeartData } from "@/features/heart/hooks";
import { useTimeRange } from "@/hooks/useTimeRange";
import { formatDate } from "@/lib/formatters";

const ECG_STATUS: Record<number, { label: string; variant: string }> = {
  0: { label: "Normal", variant: "bg-success/10 text-success-text border-success/20" },
  1: { label: "AFib", variant: "bg-destructive/10 text-destructive-text border-destructive/20" },
  2: { label: "Inconclusif", variant: "bg-warning/10 text-warning-text border-warning/20" },
  3: {
    label: "Trop court",
    variant: "bg-muted-foreground/10 text-muted-foreground border-muted-foreground/20",
  },
};

export default function Heart() {
  const { timeRange, setTimeRange, from, to } = useTimeRange();
  const { data: records, isLoading } = useHeartData(from, to);

  const totalRecordings = records?.length ?? 0;

  const normalCount = records?.filter((r) => r.ecg_status === 0).length ?? 0;
  const normalPercent = totalRecordings > 0 ? Math.round((normalCount / totalRecordings) * 100) : 0;

  const afibCount = records?.filter((r) => r.ecg_status === 1).length ?? 0;

  return (
    <div className="space-y-6">
      <TimeRangeSelector value={timeRange} onChange={setTimeRange} />

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Enregistrements</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{totalRecordings}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Normaux</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{normalCount}</p>
            <p className="text-xs text-muted-foreground mt-1">
              {normalPercent} % des enregistrements
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Alertes AFib</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{afibCount}</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Historique ECG</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="space-y-2 p-2">
              {[...Array(6)].map((_, i) => (
                <Skeleton key={i} className="h-10 w-full" />
              ))}
            </div>
          ) : records && records.length > 0 ? (
            <DataTable
              label="Historique ECG"
              columns={[
                {
                  key: "timestamp",
                  label: "Date",
                  render: (r) => formatDate(r.timestamp),
                  searchText: (r) => formatDate(r.timestamp),
                },
                {
                  key: "ecg_status",
                  label: "Statut",
                  searchText: (r) =>
                    r.ecg_status !== null ? (ECG_STATUS[r.ecg_status]?.label ?? "") : "",
                  render: (r) => {
                    const status = r.ecg_status !== null ? ECG_STATUS[r.ecg_status] : null;
                    return status ? (
                      <Badge variant="outline" className={status.variant}>
                        {status.label}
                      </Badge>
                    ) : (
                      "\u2014"
                    );
                  },
                },
                {
                  key: "heart_rate",
                  label: "FC",
                  render: (r) => (r.heart_rate ? `${r.heart_rate} bpm` : "\u2014"),
                  searchText: (r) => (r.heart_rate ? `${r.heart_rate} bpm` : ""),
                },
                {
                  key: "qrs_duration",
                  label: "QRS",
                  render: (r) => (r.qrs_duration ? `${r.qrs_duration} ms` : "\u2014"),
                  searchText: (r) => (r.qrs_duration ? `${r.qrs_duration} ms` : ""),
                },
                sourceColumn<HeartRecord>(),
              ]}
              data={records}
              searchable={false}
              emptyMessage="Aucun enregistrement ECG"
            />
          ) : (
            <PeriodEmptyState
              icon={HeartPulse}
              periodTitle="Aucun enregistrement ECG sur cette période"
              noneTitle="Aucun enregistrement ECG"
              noneDescription="Les ECG de vos appareils apparaissent ici après synchronisation."
              timeRange={timeRange}
              onShowAll={() => setTimeRange("all")}
            />
          )}
        </CardContent>
      </Card>

      <div className="flex items-start gap-2 rounded-md border bg-muted/50 p-3">
        <Info className="h-4 w-4 mt-0.5 shrink-0 text-muted-foreground" />
        <p className="text-sm text-muted-foreground">
          Le signal ECG complet est visible dans l'application Withings
        </p>
      </div>
    </div>
  );
}
