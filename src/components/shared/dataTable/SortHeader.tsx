import { ArrowUpDown, ArrowUp, ArrowDown } from "lucide-react";
import type { SortDir } from "./types";

interface Props {
  label: string;
  active: boolean;
  sortDir: SortDir;
  onSort: () => void;
}

function SortIcon({ active, sortDir }: { active: boolean; sortDir: SortDir }) {
  if (!active) return <ArrowUpDown className="h-3.5 w-3.5 opacity-30" />;
  return sortDir === "asc" ? (
    <ArrowUp className="h-3.5 w-3.5" />
  ) : (
    <ArrowDown className="h-3.5 w-3.5" />
  );
}

export function SortHeader({ label, active, sortDir, onSort }: Props) {
  return (
    <th
      role="columnheader"
      tabIndex={0}
      aria-sort={active ? (sortDir === "asc" ? "ascending" : "descending") : "none"}
      className="px-4 py-3 text-left font-medium text-muted-foreground cursor-pointer select-none hover:text-foreground transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
      onClick={onSort}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onSort();
        }
      }}
    >
      <span className="inline-flex items-center gap-1">
        {label}
        <SortIcon active={active} sortDir={sortDir} />
      </span>
    </th>
  );
}
