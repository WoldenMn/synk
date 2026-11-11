import { useState } from "react";

const seen = new Set<string>();

export function useEntranceOnce(key: string): boolean {
  const [first] = useState(() => {
    const isFirst = !seen.has(key);
    seen.add(key);
    return isFirst;
  });
  return first;
}
