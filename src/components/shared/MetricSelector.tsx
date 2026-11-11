import { METRIC_CATEGORY_LABELS } from "@/features/metrics/categoryLabels";
import type { MetricDefinition } from "@/features/metrics/types";
import { cn } from "@/lib/utils";
import { Select } from "@/components/ui/select";

interface MetricSelectorProps {
  definitions: MetricDefinition[];
  value: string | null;
  onChange: (slug: string) => void;
  placeholder?: string;
  className?: string;
}

export function MetricSelector({
  definitions,
  value,
  onChange,
  placeholder = "Choisir une métrique...",
  className,
}: MetricSelectorProps) {
  const grouped = definitions.reduce<Record<string, MetricDefinition[]>>((acc, def) => {
    const cat = def.category;
    if (!acc[cat]) acc[cat] = [];
    acc[cat].push(def);
    return acc;
  }, {});

  return (
    <Select
      value={value ?? ""}
      onChange={(e) => onChange(e.target.value)}
      className={cn(
        "h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm",
        "ring-offset-background focus:outline-hidden focus:ring-2 focus:ring-ring focus:ring-offset-2",
        className
      )}
      aria-label={placeholder}
    >
      <option value="" disabled>
        {placeholder}
      </option>
      {Object.entries(grouped).map(([cat, defs]) => (
        <optgroup key={cat} label={(METRIC_CATEGORY_LABELS as Record<string, string>)[cat] ?? cat}>
          {defs.map((d) => (
            <option key={d.slug} value={d.slug}>
              {d.label}
              {d.unit ? ` (${d.unit})` : ""}
            </option>
          ))}
        </optgroup>
      ))}
    </Select>
  );
}
