import { FileBraces, Upload } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

interface Props {
  exporting: boolean;
  handleExportJson: () => void;
  handleImportJson: () => void;
}

export function JsonCard({ exporting, handleExportJson, handleImportJson }: Props) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <FileBraces className="h-5 w-5" />
          Export complet (JSON)
        </CardTitle>
        <CardDescription>
          Toutes vos données de santé, dans un format lisible par d'autres outils. Pour une copie
          restaurable, voir Paramètres &gt; Sauvegardes.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex gap-3">
          <Button className="flex-1" disabled={exporting} onClick={handleExportJson}>
            <FileBraces className="mr-2 h-4 w-4" />
            Exporter JSON
          </Button>
          <Button
            variant="outline"
            className="flex-1"
            disabled={exporting}
            onClick={handleImportJson}
          >
            <Upload className="mr-2 h-4 w-4" />
            Importer JSON
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
