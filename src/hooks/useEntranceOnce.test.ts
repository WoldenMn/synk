import { describe, expect, it } from "vitest";
import { renderHook } from "@testing-library/react";
import { useEntranceOnce } from "./useEntranceOnce";

describe("useEntranceOnce", () => {
  it("anime au premier montage d'une clé, plus ensuite", () => {
    expect(renderHook(() => useEntranceOnce("test-a")).result.current).toBe(true);
    expect(renderHook(() => useEntranceOnce("test-a")).result.current).toBe(false);
    expect(renderHook(() => useEntranceOnce("test-b")).result.current).toBe(true);
  });

  it("ne change pas d'avis au rendu suivant du même composant", () => {
    const { result, rerender } = renderHook(() => useEntranceOnce("test-c"));
    rerender();
    expect(result.current).toBe(true);
  });
});
