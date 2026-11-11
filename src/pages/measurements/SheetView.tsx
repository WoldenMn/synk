import { useState } from "react";
import { FileDown, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { LoadingSkeleton } from "@/components/shared/LoadingSkeleton";
import { useBodySheet, useExportBodySheetPdf } from "@/features/bodyProfile/hooks";
import { MeasurementWizard } from "@/features/bodyProfile/components/MeasurementWizard";
import { BodyFigure } from "@/features/bodyProfile/components/BodyFigure";
import { KEY_MEASUREMENTS, ZONES, findMeasurement } from "@/features/bodyProfile/catalog";

export function SheetView() {
  const [wizardOpen, setWizardOpen] = useState(false);
  const { data: rows, isLoading } = useBodySheet();
  const exportPdf = useExportBodySheetPdf();

  if (isLoading) return <LoadingSkeleton rows={8} height="h-10" />;

  return (
    <div className="space-y-6">
      <MeasurementWizard open={wizardOpen} onOpenChange={setWizardOpen} />

      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">Fiche de mensurations</h2>
          <p className="text-sm text-muted-foreground mt-1">
            Vos mesures de référence pour le sur-mesure
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => exportPdf.mutate()}
            disabled={exportPdf.isPending}
          >
            <FileDown className="mr-2 h-4 w-4" />
            Exporter en PDF
          </Button>
          <Button size="sm" onClick={() => setWizardOpen(true)}>
            <Plus className="mr-2 h-4 w-4" />
            Nouveau relevé
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-[1fr_2fr] gap-8">
        <div className="grid grid-cols-2 gap-2">
          <BodyFigure view="front" annotations={KEY_MEASUREMENTS} />
          <BodyFigure view="back" annotations={KEY_MEASUREMENTS} />
        </div>

        <div className="space-y-6">
          {ZONES.map((zone) => {
            const zoneRows = (rows ?? []).filter((r) => findMeasurement(r.slug)?.zone === zone.id);
            if (zoneRows.length === 0) return null;
            return (
              <div key={zone.id}>
                <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-2">
                  {zone.label}
                </h2>
                <table className="w-full text-sm">
                  <tbody className="[&>:not([hidden])~:not([hidden])]:border-t">
                    {zoneRows.map((r) => (
                      <tr key={r.slug}>
                        <td className="py-1.5">{r.label}</td>
                        <td className="py-1.5 text-right font-medium">
                          {r.value != null ? r.value : "—"}
                        </td>
                        <td className="py-1.5 pl-2 text-muted-foreground w-10">{r.unit}</td>
                        <td className="py-1.5 pl-4 text-xs text-muted-foreground w-24">
                          {r.measured_at ?? ""}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
