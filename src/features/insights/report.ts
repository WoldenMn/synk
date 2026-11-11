import type { InsightReport } from "./api";

export function describeInsightReport(report: InsightReport): {
  success: string;
  failure: string | null;
} {
  const n = report.generated;
  const success =
    n === 0 ? "Aucun nouvel insight" : `${n} insight${n > 1 ? "s" : ""} généré${n > 1 ? "s" : ""}`;
  const failure =
    report.failed.length === 0 ? null : `Analyses en échec : ${report.failed.join(", ")}`;
  return { success, failure };
}
