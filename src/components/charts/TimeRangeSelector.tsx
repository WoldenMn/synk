import { useState } from "react";
import { ArrowLeftRight } from "lucide-react";
import { TIME_RANGES } from "@/lib/constants";
import { cn } from "@/lib/utils";
import type { TimeRangeValue } from "@/lib/timeRange";
import { Input } from "@/components/ui/input";

interface TimeRangeSelectorProps {
  value: TimeRangeValue;
  onChange: (value: TimeRangeValue) => void;
  compareFrom?: string | null;
  compareTo?: string | null;
  onCompareChange?: (from: string | null, to: string | null) => void;
}

export function TimeRangeSelector({
  value,
  onChange,
  compareFrom,
  compareTo,
  onCompareChange,
}: TimeRangeSelectorProps) {
  const [compareActive, setCompareActive] = useState(
    () => compareFrom != null && compareTo != null
  );

  function handleToggle() {
    const next = !compareActive;
    setCompareActive(next);
    if (!next) {
      onCompareChange?.(null, null);
    }
  }

  function handleFromChange(e: React.ChangeEvent<HTMLInputElement>) {
    onCompareChange?.(e.target.value || null, compareTo ?? null);
  }

  function handleToChange(e: React.ChangeEvent<HTMLInputElement>) {
    onCompareChange?.(compareFrom ?? null, e.target.value || null);
  }

  return (
    <div className="flex flex-col items-end gap-2">
      <div className="inline-flex rounded-lg border bg-muted p-1">
        {TIME_RANGES.map((range) => (
          <button
            key={range.value}
            type="button"
            aria-pressed={value === range.value}
            onClick={() => onChange(range.value)}
            className={cn(
              "rounded-md px-3 py-1.5 text-sm font-medium transition-colors",
              value === range.value
                ? "bg-background text-foreground shadow-xs"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            {range.label}
          </button>
        ))}
        {onCompareChange && (
          <button
            type="button"
            aria-pressed={compareActive}
            onClick={handleToggle}
            className={cn(
              "ml-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors inline-flex items-center gap-1.5",
              compareActive
                ? "bg-primary text-primary-foreground shadow-xs"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            <ArrowLeftRight className="h-3.5 w-3.5" />
            Comparer
          </button>
        )}
      </div>
      {compareActive && onCompareChange && (
        <div className="inline-flex items-center gap-2 rounded-lg border bg-muted/50 px-3 py-2 text-sm">
          <span className="text-muted-foreground">Du</span>
          <Input
            type="date"
            aria-label="Date de début"
            value={compareFrom ?? ""}
            onChange={handleFromChange}
            className="rounded-md border bg-background px-2 py-1 text-sm"
          />
          <span className="text-muted-foreground">Au</span>
          <Input
            type="date"
            aria-label="Date de fin"
            value={compareTo ?? ""}
            onChange={handleToChange}
            className="rounded-md border bg-background px-2 py-1 text-sm"
          />
        </div>
      )}
    </div>
  );
}
