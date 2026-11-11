import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { MeasurementProgressChart } from "@/components/charts/MeasurementProgressChart";
import { TimeRangeSelector } from "@/components/charts/TimeRangeSelector";
import {
  useManualMeasurements,
  useMeasurementStats,
  useDeleteManualMeasurement,
} from "@/features/manualMeasurements/hooks";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { useTimeRange } from "@/hooks/useTimeRange";
import type { MeasurementType } from "@/features/manualMeasurements/types";
import { DetailHeader } from "./detail/DetailHeader";
import { DetailStats } from "./detail/DetailStats";
import { DetailHistory } from "./detail/DetailHistory";

interface MeasurementDetailProps {
  typeId: number;
  types: MeasurementType[];
  onBack: () => void;
  onAddEntry: () => void;
  onOpenGoal: () => void;
}

export function MeasurementDetail({
  typeId,
  types,
  onBack,
  onAddEntry,
  onOpenGoal,
}: MeasurementDetailProps) {
  const { timeRange, setTimeRange, from, to } = useTimeRange();
  const selectedType = types.find((t) => t.id === typeId);
  const { data: measurements, isLoading } = useManualMeasurements(typeId, from, to);
  const { data: stats } = useMeasurementStats(typeId, from, to);
  const deleteMutation = useDeleteManualMeasurement();
  const confirm = useConfirm();

  async function handleDelete(id: number) {
    if (!(await confirm({ title: "Supprimer cette mesure ?", destructive: true }))) return;
    deleteMutation.mutate(id);
  }

  return (
    <div className="space-y-6">
      <DetailHeader
        selectedType={selectedType}
        onBack={onBack}
        onAddEntry={onAddEntry}
        onOpenGoal={onOpenGoal}
      />

      <TimeRangeSelector value={timeRange} onChange={setTimeRange} />

      {stats && <DetailStats stats={stats} selectedType={selectedType} />}

      {measurements && measurements.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle as="h3">Progression</CardTitle>
          </CardHeader>
          <CardContent>
            <MeasurementProgressChart
              data={measurements}
              unit={selectedType?.unit || ""}
              goalValue={selectedType?.target_value}
            />
          </CardContent>
        </Card>
      )}

      <DetailHistory
        measurements={measurements}
        isLoading={isLoading}
        selectedType={selectedType}
        onDelete={handleDelete}
      />
    </div>
  );
}
