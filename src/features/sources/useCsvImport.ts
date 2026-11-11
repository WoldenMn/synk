import { useCallback, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";
import { deleteCsvMapping, importCsvFile, inspectCsvFile, saveCsvMapping } from "./csv/api";
import { useImportRunner } from "./useImportRunner";
import type { CsvConfig, CsvInspection, SourceInfo } from "./types";

export type CsvStep =
  | { step: "choose" }
  | { step: "inspecting" }
  | { step: "assistant"; path: string; inspection: CsvInspection };

const CONTEXT = "import_csv_file";

export function useCsvImport(source: SourceInfo) {
  const { state, run, cancel, reset } = useImportRunner(CONTEXT);
  const [flow, setFlow] = useState<CsvStep>({ step: "choose" });

  const pick = useCallback(async () => {
    try {
      const file = await openDialog({
        title: `Importer un fichier ${source.label}`,
        filters: [{ name: source.label, extensions: source.extensions }],
        multiple: false,
      });
      if (typeof file !== "string") return;
      reset();
      setFlow({ step: "inspecting" });
      const inspection = await inspectCsvFile(file);
      setFlow({ step: "assistant", path: file, inspection });
    } catch (e) {
      setFlow({ step: "choose" });
      reportError(e, { source: "manual", context: "inspect_csv_file" });
    }
  }, [reset, source.extensions, source.label]);

  const start = useCallback(
    async (config: CsvConfig) => {
      if (flow.step !== "assistant") return;
      const report = await run(() => importCsvFile(flow.path, config));
      if (report) setFlow({ step: "choose" });
    },
    [flow, run]
  );

  const save = useCallback(
    async (name: string, config: CsvConfig) => {
      if (flow.step !== "assistant") return;
      try {
        const id = await saveCsvMapping(name, flow.inspection.signature, config);
        const saved = [
          { id, name: name.trim(), config },
          ...flow.inspection.saved.filter((m) => m.id !== id),
        ];
        setFlow({ ...flow, inspection: { ...flow.inspection, saved } });
        toast.success("Correspondance enregistrée.");
      } catch (e) {
        reportError(e, { source: "manual", context: "save_csv_mapping" });
      }
    },
    [flow]
  );

  const forget = useCallback(
    async (id: number) => {
      if (flow.step !== "assistant") return;
      try {
        await deleteCsvMapping(id);
        const saved = flow.inspection.saved.filter((m) => m.id !== id);
        setFlow({ ...flow, inspection: { ...flow.inspection, saved } });
      } catch (e) {
        reportError(e, { source: "manual", context: "delete_csv_mapping" });
      }
    },
    [flow]
  );

  const close = useCallback(() => setFlow({ step: "choose" }), []);

  return { state, flow, pick, start, save, forget, close, cancel };
}
