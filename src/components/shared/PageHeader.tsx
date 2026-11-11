import { Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { TimeRangeSelector } from "@/components/charts/TimeRangeSelector";
import type { TimeRangeValue } from "@/lib/timeRange";

interface PageHeaderProps {
  timeRange: TimeRangeValue;
  onTimeRangeChange: (value: TimeRangeValue) => void;
  onAdd?: () => void;
  addLabel?: string;
  children?: React.ReactNode;
}

export function PageHeader({
  timeRange,
  onTimeRangeChange,
  onAdd,
  addLabel = "Ajouter",
  children,
}: PageHeaderProps) {
  return (
    <div className="flex items-center justify-between gap-4 flex-wrap">
      <TimeRangeSelector value={timeRange} onChange={onTimeRangeChange} />
      <div className="flex items-center gap-2">
        {children}
        {onAdd && (
          <Button size="sm" onClick={onAdd}>
            <Plus className="mr-2 h-4 w-4" />
            {addLabel}
          </Button>
        )}
      </div>
    </div>
  );
}
