import { useState, useMemo } from "react";
import type { Column, SortDir } from "./types";

function compareValues(av: unknown, bv: unknown, sortDir: SortDir): number {
  if (av == null && bv == null) return 0;
  if (av == null) return 1;
  if (bv == null) return -1;
  const aStr = String(av);
  const bStr = String(bv);
  const aNum = Number(av);
  const bNum = Number(bv);
  let cmp: number;
  if (!isNaN(aNum) && !isNaN(bNum)) {
    cmp = aNum - bNum;
  } else {
    cmp = aStr.localeCompare(bStr);
  }
  return sortDir === "asc" ? cmp : -cmp;
}

export function useDataTableState<T extends { id: number }>(
  data: T[],
  columns: Column<T>[],
  pageSize: number
) {
  const [search, setSearch] = useState("");
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortDir, setSortDir] = useState<SortDir>("asc");
  const [page, setPage] = useState(0);

  const filtered = useMemo(() => {
    if (!search.trim()) return data;
    const q = search.toLowerCase();
    return data.filter((row) =>
      columns.some((col) => {
        const val = col.searchText
          ? col.searchText(row)
          : col.render
            ? undefined
            : (row as Record<string, unknown>)[col.key];
        if (val == null) return false;
        return String(val).toLowerCase().includes(q);
      })
    );
  }, [data, search, columns]);

  const sorted = useMemo(() => {
    if (!sortKey) return filtered;
    return [...filtered].sort((a, b) =>
      compareValues(
        (a as Record<string, unknown>)[sortKey],
        (b as Record<string, unknown>)[sortKey],
        sortDir
      )
    );
  }, [filtered, sortKey, sortDir]);

  const totalPages = Math.max(1, Math.ceil(sorted.length / pageSize));
  const safePage = Math.min(page, totalPages - 1);
  const paged = sorted.slice(safePage * pageSize, (safePage + 1) * pageSize);

  function handleSort(key: string) {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
    setPage(0);
  }

  function handleSearch(val: string) {
    setSearch(val);
    setPage(0);
  }

  return {
    search,
    sortKey,
    sortDir,
    sorted,
    paged,
    totalPages,
    safePage,
    handleSort,
    handleSearch,
    previousPage: () => setPage((p) => p - 1),
    nextPage: () => setPage((p) => p + 1),
  };
}
