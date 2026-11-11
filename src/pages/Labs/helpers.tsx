import { TrendingUp, TrendingDown, CircleCheck, TriangleAlert } from "lucide-react";
import type { LabResult } from "@/features/labs/types";

export function isInRange(value: number, min: number | null, max: number | null): boolean | null {
  if (min == null && max == null) return null;
  if (min != null && value < min) return false;
  if (max != null && value > max) return false;
  return true;
}

export function StatusDot({ inRange }: { inRange: boolean | null }) {
  if (inRange === null) {
    return (
      <span className="inline-block h-2.5 w-2.5 rounded-full bg-muted-foreground/30">
        <span className="sr-only">sans référence</span>
      </span>
    );
  }
  return inRange ? (
    <span className="inline-flex text-success-text">
      <CircleCheck className="h-3.5 w-3.5" />
      <span className="sr-only">dans la référence</span>
    </span>
  ) : (
    <span className="inline-flex text-destructive-text">
      <TriangleAlert className="h-3.5 w-3.5" />
      <span className="sr-only">hors référence</span>
    </span>
  );
}

export function TrendArrow({ items }: { items: LabResult[] }) {
  if (items.length < 2) return null;
  const latest = items[0]?.value;
  const prev = items[1]?.value;
  if (latest === undefined || prev === undefined) return null;
  if (latest > prev) {
    return (
      <span className="inline-flex text-muted-foreground">
        <TrendingUp className="h-3.5 w-3.5" />
        <span className="sr-only">en hausse</span>
      </span>
    );
  }
  if (latest < prev) {
    return (
      <span className="inline-flex text-muted-foreground">
        <TrendingDown className="h-3.5 w-3.5" />
        <span className="sr-only">en baisse</span>
      </span>
    );
  }
  return null;
}

export function MiniLabChart({ items }: { items: LabResult[] }) {
  if (items.length < 2) return null;
  const values = [...items].reverse().map((i) => i.value);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  const h = 28;
  const w = 64;
  const points = values
    .map((v, i) => `${(i / (values.length - 1)) * w},${h - ((v - min) / range) * h}`)
    .join(" ");

  const refMin = items[0]?.reference_min;
  const refMax = items[0]?.reference_max;

  return (
    <svg width={w} height={h} className="shrink-0">
      {refMin != null && refMax != null && (
        <rect
          x={0}
          y={h - ((refMax - min) / range) * h}
          width={w}
          height={Math.max(1, ((refMax - refMin) / range) * h)}
          fill="hsl(var(--primary))"
          opacity={0.08}
          rx={2}
        />
      )}
      <polyline
        points={points}
        fill="none"
        stroke="hsl(var(--primary))"
        strokeWidth={1.5}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
