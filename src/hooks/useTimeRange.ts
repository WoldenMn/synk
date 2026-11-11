import { useState, useMemo, useCallback } from "react";
import { subDays, formatISO, startOfDay, endOfDay } from "date-fns";
import type { TimeRangeValue } from "@/lib/timeRange";
import { TIME_RANGES } from "@/lib/constants";

export function rangeBounds(timeRange: TimeRangeValue, now: Date = new Date()) {
  const range = TIME_RANGES.find((r) => r.value === timeRange);
  const days = range?.days ?? 30;
  return {
    from: formatISO(startOfDay(subDays(now, days))),
    to: formatISO(endOfDay(now)),
  };
}

export function useTimeRange(defaultValue: TimeRangeValue = "30d") {
  const [timeRange, setTimeRange] = useState<TimeRangeValue>(defaultValue);
  const [compareFrom, setCompareFrom] = useState<string | null>(null);
  const [compareTo, setCompareTo] = useState<string | null>(null);

  const { from, to } = useMemo(() => rangeBounds(timeRange), [timeRange]);

  const setCompareRange = useCallback((from: string | null, to: string | null) => {
    setCompareFrom(from);
    setCompareTo(to);
  }, []);

  return { timeRange, setTimeRange, from, to, compareFrom, compareTo, setCompareRange };
}
