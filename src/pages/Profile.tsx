import { FileDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import { LoadingSkeleton } from "@/components/shared/LoadingSkeleton";
import { useProfileCard, useExportProfileCardPdf } from "@/features/profileCard/hooks";
import { BodySection } from "./profile/sections/BodySection";
import { MeasurementsSection } from "./profile/sections/MeasurementsSection";
import { HealthSection } from "./profile/sections/HealthSection";
import { SizesSection } from "./profile/sections/SizesSection";
import { RecordsSection } from "./profile/sections/RecordsSection";

export default function Profile() {
  const { data: card, isLoading } = useProfileCard();
  const exportPdf = useExportProfileCardPdf();

  if (isLoading || !card) return <LoadingSkeleton rows={8} height="h-10" />;

  return (
    <div className="space-y-8">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">Votre état des lieux chiffré</p>
        <Button
          variant="outline"
          size="sm"
          onClick={() => exportPdf.mutate()}
          disabled={exportPdf.isPending}
        >
          <FileDown className="mr-2 h-4 w-4" />
          Exporter en PDF
        </Button>
      </div>

      <div className="grid items-start gap-x-12 gap-y-8 lg:grid-cols-2">
        <div className="flex flex-col gap-8">
          <BodySection body={card.body} />
          <HealthSection health={card.health} />
          <RecordsSection records={card.records} />
        </div>
        <div className="flex flex-col gap-8">
          <MeasurementsSection measurements={card.key_measurements} />
          <SizesSection sizes={card.sizes} />
        </div>
      </div>
    </div>
  );
}
