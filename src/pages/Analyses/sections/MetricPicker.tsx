import { Zap } from "lucide-react";
import { Select } from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { METRIC_OPTIONS, PRESETS } from "../helpers";

interface Props {
  metric1: string;
  metric2: string;
  setMetric1: (v: string) => void;
  setMetric2: (v: string) => void;
}

export function MetricPicker({ metric1, metric2, setMetric1, setMetric2 }: Props) {
  return (
    <div className="flex flex-wrap gap-4 items-end">
      <div className="space-y-1.5">
        <label htmlFor="analyses-metric-1" className="text-sm font-medium text-muted-foreground">
          Métrique 1
        </label>
        <Select
          id="analyses-metric-1"
          value={metric1}
          onChange={(e) => setMetric1(e.target.value)}
          className="w-48"
        >
          {METRIC_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label} ({opt.unit})
            </option>
          ))}
        </Select>
      </div>

      <div className="space-y-1.5">
        <label htmlFor="analyses-metric-2" className="text-sm font-medium text-muted-foreground">
          Métrique 2
        </label>
        <Select
          id="analyses-metric-2"
          value={metric2}
          onChange={(e) => setMetric2(e.target.value)}
          className="w-48"
        >
          {METRIC_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label} ({opt.unit})
            </option>
          ))}
        </Select>
      </div>

      <div className="flex gap-2 flex-wrap">
        {PRESETS.map((preset) => (
          <Button
            key={preset.label}
            variant="outline"
            size="sm"
            onClick={() => {
              setMetric1(preset.metric1);
              setMetric2(preset.metric2);
            }}
            className="gap-1.5"
          >
            <Zap className="h-3.5 w-3.5" />
            {preset.label}
          </Button>
        ))}
      </div>
    </div>
  );
}
