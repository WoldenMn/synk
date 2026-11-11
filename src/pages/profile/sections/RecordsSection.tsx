import type { RecordEntry } from "@/features/profileCard/types";
import { formatProfileDate, formatRecordValue } from "@/features/profileCard/format";

export function RecordsSection({ records }: { records: RecordEntry[] }) {
  return (
    <section>
      <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
        Records
      </h2>
      {records.length === 0 ? (
        <p className="text-sm text-muted-foreground">Pas encore assez d'historique.</p>
      ) : (
        <dl className="grid grid-cols-2 gap-x-8 gap-y-1.5 text-sm">
          {records.map((r) => (
            <div key={r.key} className="flex justify-between border-b py-1">
              <dt className="text-muted-foreground">{r.label}</dt>
              <dd className="font-medium tabular-nums text-right">
                {formatRecordValue(r.value, r.unit)}
                <span className="text-xs text-muted-foreground ml-2">
                  {formatProfileDate(r.achieved_at)}
                </span>
              </dd>
            </div>
          ))}
        </dl>
      )}
    </section>
  );
}
