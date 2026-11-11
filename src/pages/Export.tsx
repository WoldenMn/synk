import { ExportCard } from "./export/ExportCard";
import { JsonCard } from "./export/JsonCard";
import { useExportState } from "./export/useExportState";

export default function Export() {
  const state = useExportState();

  return (
    <div className="space-y-6">
      <p className="text-sm text-muted-foreground">
        Les fichiers exportés ne sont pas chiffrés : ils contiennent vos données de santé en clair.
        Rangez-les dans un dossier protégé, et supprimez ceux dont vous n'avez plus besoin.
      </p>
      <div className="grid grid-cols-1 items-start gap-6 xl:grid-cols-2">
        <ExportCard {...state} />
        <JsonCard
          exporting={state.exporting}
          handleExportJson={state.handleExportJson}
          handleImportJson={state.handleImportJson}
        />
      </div>
    </div>
  );
}
