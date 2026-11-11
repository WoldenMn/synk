import { useCallback } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { reportError } from "@/lib/reportError";
import { importSourceFile } from "./api";
import { useImportRunner } from "./useImportRunner";
import type { SourceInfo } from "./types";

export type { ImportState } from "./useImportRunner";

export function useImportSource(source: SourceInfo) {
  const { state, run, cancel } = useImportRunner(`import_source_file:${source.id}`);

  const start = useCallback(async () => {
    try {
      const file = await openDialog({
        title: `Importer un export ${source.label}`,
        filters: [{ name: source.label, extensions: source.extensions }],
        multiple: false,
      });
      if (typeof file !== "string") return;
      await run(() => importSourceFile(source.id, file));
    } catch (e) {
      reportError(e, { source: "manual", context: `import_source_file:${source.id}` });
    }
  }, [run, source.extensions, source.id, source.label]);

  return { state, start, cancel };
}
