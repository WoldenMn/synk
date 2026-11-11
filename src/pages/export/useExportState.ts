import { useState } from "react";
import {
  exportCsv,
  exportPdf,
  exportJson,
  importJson,
  inspectImportFile,
} from "@/features/export/api";
import { describePreview, summarizeImport } from "@/features/export/importSummary";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";
import { format as formatDay, subMonths } from "date-fns";
import { nowDate } from "@/lib/dates";

export const CATEGORIES = [
  { key: "body_measures", label: "Composition corporelle" },
  { key: "blood_pressure", label: "Tension artérielle" },
  { key: "sleep", label: "Sommeil" },
  { key: "activity", label: "Activité" },
  { key: "workouts", label: "Entraînements" },
  { key: "heart", label: "Cœur / ECG" },
  { key: "measurements", label: "Mensurations" },
] as const;

const ALL_HISTORY_FROM = "1970-01-01T00:00:00.000Z";

export function useExportState() {
  const confirm = useConfirm();
  const [allHistory, setAllHistory] = useState(true);
  const [selectedCategories, setSelectedCategories] = useState<Set<string>>(
    new Set(CATEGORIES.map((c) => c.key))
  );
  const [dateFrom, setDateFrom] = useState(() => formatDay(subMonths(new Date(), 3), "yyyy-MM-dd"));
  const [dateTo, setDateTo] = useState(nowDate);
  const [exporting, setExporting] = useState(false);
  const [lastExportPath, setLastExportPath] = useState<string | null>(null);

  function toggleCategory(key: string) {
    setSelectedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  async function handleExport(format: "csv" | "pdf") {
    setExporting(true);
    setLastExportPath(null);
    try {
      const cats = Array.from(selectedCategories);
      const from = allHistory ? ALL_HISTORY_FROM : new Date(dateFrom).toISOString();
      const toDay = allHistory ? nowDate() : dateTo;
      const to = new Date(toDay + "T23:59:59").toISOString();
      const outcome =
        format === "csv" ? await exportCsv(from, to, cats) : await exportPdf(from, to, cats);
      if (outcome.rows === 0 || outcome.path === null) {
        toast.info("Aucune donnée sur cette période : rien n'a été exporté.");
        return;
      }
      setLastExportPath(outcome.path);
      const rows = `${outcome.rows} ${outcome.rows > 1 ? "lignes" : "ligne"}`;
      toast.success(`${format === "csv" ? "CSV" : "PDF"} exporté : ${rows}`, {
        description: outcome.path,
      });
    } catch (e) {
      reportError(e, { source: "manual", context: "export" });
    } finally {
      setExporting(false);
    }
  }

  async function handleOpenFolder() {
    if (!lastExportPath) return;
    try {
      await revealItemInDir(lastExportPath);
    } catch (e) {
      reportError(e, { source: "manual", context: "export_reveal" });
    }
  }

  async function handleExportJson() {
    setExporting(true);
    try {
      const path = await exportJson();
      setLastExportPath(path);
      toast.success("JSON exporté", { description: path });
    } catch (e) {
      reportError(e, { source: "manual", context: "export_json" });
    } finally {
      setExporting(false);
    }
  }

  async function handleImportJson() {
    setExporting(true);
    try {
      const file = await openDialog({
        title: "Importer un fichier JSON",
        filters: [{ name: "JSON", extensions: ["json"] }],
        multiple: false,
      });
      if (!file) return;
      const path = file as string;
      const preview = await inspectImportFile(path);
      const accepted = await confirm({
        title: "Importer ce fichier ?",
        description: describePreview(preview),
        confirmLabel: "Importer",
        destructive: false,
      });
      if (!accepted) return;
      const report = await importJson(path);
      toast.success(summarizeImport(report.tables));
      const orphans = report.tables.reduce((n, t) => n + t.orphans, 0);
      if (orphans > 0) {
        toast.warning(
          `${orphans} ligne(s) ignorée(s) : leur élément parent est absent du fichier.`
        );
      }
    } catch (e) {
      reportError(e, { source: "manual", context: "import_json" });
    } finally {
      setExporting(false);
    }
  }

  return {
    allHistory,
    setAllHistory,
    selectedCategories,
    toggleCategory,
    dateFrom,
    setDateFrom,
    dateTo,
    setDateTo,
    exporting,
    lastExportPath,
    handleExport,
    handleOpenFolder,
    handleExportJson,
    handleImportJson,
  };
}

export type ExportState = ReturnType<typeof useExportState>;
