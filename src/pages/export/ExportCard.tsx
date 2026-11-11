import { Download, FileText, Table, FolderOpen } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { CATEGORIES, type ExportState } from "./useExportState";
import { Input } from "@/components/ui/input";

export function ExportCard({
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
}: ExportState) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Download className="h-5 w-5" />
          Exporter les données
        </CardTitle>
        <CardDescription>Sélectionnez les catégories et la période à exporter</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-3">
          <p className="text-sm font-medium" id="export-period-label">
            Période
          </p>
          <div className="flex items-center gap-3">
            <Input
              id="export-date-from"
              type="date"
              aria-label="Date de début"
              value={dateFrom}
              disabled={allHistory}
              onChange={(e) => setDateFrom(e.target.value)}
              className="rounded-md border bg-background px-3 py-2 text-sm disabled:opacity-50"
            />
            <span className="text-sm text-muted-foreground">au</span>
            <Input
              id="export-date-to"
              type="date"
              aria-label="Date de fin"
              value={dateTo}
              disabled={allHistory}
              onChange={(e) => setDateTo(e.target.value)}
              className="rounded-md border bg-background px-3 py-2 text-sm disabled:opacity-50"
            />
          </div>
          <label className="flex cursor-pointer items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={allHistory}
              onChange={(e) => setAllHistory(e.target.checked)}
              className="h-4 w-4 rounded"
            />
            Tout l'historique
          </label>
        </div>

        <div className="space-y-3">
          <p className="text-sm font-medium">Catégories</p>
          <div className="grid gap-2 sm:grid-cols-2">
            {CATEGORIES.map((cat) => (
              <label
                key={cat.key}
                className="flex cursor-pointer items-center gap-2 rounded-md border p-3 hover:bg-accent/15 transition-colors"
              >
                <input
                  type="checkbox"
                  checked={selectedCategories.has(cat.key)}
                  onChange={() => toggleCategory(cat.key)}
                  className="h-4 w-4 rounded"
                />
                <span className="text-sm">{cat.label}</span>
              </label>
            ))}
          </div>
        </div>

        <div className="flex gap-3">
          <Button
            onClick={() => handleExport("csv")}
            disabled={exporting || selectedCategories.size === 0}
            variant="outline"
            className="flex-1"
          >
            <Table className="mr-2 h-4 w-4" />
            Exporter CSV
          </Button>
          <Button
            onClick={() => handleExport("pdf")}
            disabled={exporting || selectedCategories.size === 0}
            className="flex-1"
          >
            <FileText className="mr-2 h-4 w-4" />
            Exporter PDF
          </Button>
        </div>

        {lastExportPath && (
          <div className="flex items-center gap-3 rounded-md border bg-muted p-3">
            <Tooltip content={lastExportPath} side="top" className="flex-1 min-w-0">
              <p className="text-sm font-mono truncate w-full">{lastExportPath}</p>
            </Tooltip>
            <Button variant="outline" size="sm" onClick={handleOpenFolder}>
              <FolderOpen className="mr-2 h-4 w-4" />
              Ouvrir
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
