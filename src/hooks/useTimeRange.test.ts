import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useTimeRange } from "./useTimeRange";

describe("useTimeRange", () => {
  it("utilise la valeur par defaut '30d'", () => {
    const { result } = renderHook(() => useTimeRange());
    expect(result.current.timeRange).toBe("30d");
  });

  it("respecte la valeur initiale passee en paramètre", () => {
    const { result } = renderHook(() => useTimeRange("7d"));
    expect(result.current.timeRange).toBe("7d");
  });

  it("met a jour from et to quand setTimeRange est appele", () => {
    const { result } = renderHook(() => useTimeRange("30d"));
    const fromInitial = result.current.from;

    act(() => {
      result.current.setTimeRange("7d");
    });

    expect(result.current.timeRange).toBe("7d");
    expect(result.current.from).not.toBe(fromInitial);
  });

  it("permet de définir une plage de comparaison", () => {
    const { result } = renderHook(() => useTimeRange());

    act(() => {
      result.current.setCompareRange("2026-01-01", "2026-01-31");
    });

    expect(result.current.compareFrom).toBe("2026-01-01");
    expect(result.current.compareTo).toBe("2026-01-31");
  });
  it("garde la même borne de fin d'un montage à l'autre dans la journée", () => {
    vi.useFakeTimers({ toFake: ["Date"] });
    try {
      vi.setSystemTime(new Date(2026, 9, 3, 9, 0, 0));
      const first = renderHook(() => useTimeRange()).result.current.to;
      vi.setSystemTime(new Date(2026, 9, 3, 16, 42, 7));
      const second = renderHook(() => useTimeRange()).result.current.to;
      expect(second).toBe(first);
      expect(first.startsWith("2026-10-03T23:59:59")).toBe(true);
    } finally {
      vi.useRealTimers();
    }
  });
});
