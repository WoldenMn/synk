import { Search } from "lucide-react";
import { PaginationBar } from "./dataTable/PaginationBar";
import { SortHeader } from "./dataTable/SortHeader";
import { TableRows } from "./dataTable/TableRows";
import type { Column } from "./dataTable/types";
import { useDataTableState } from "./dataTable/useDataTableState";
import { Input } from "@/components/ui/input";

interface DataTableBaseProps<T extends { id: number }> {
  label: string;
  columns: Column<T>[];
  data: T[];
  canDelete?: (row: T) => boolean;
  emptyMessage: string;
  searchable?: boolean;
  pageSize?: number;
}

type DataTableProps<T extends { id: number }> = DataTableBaseProps<T> &
  (
    | { onDelete: (id: number) => void; rowLabel: (row: T) => string }
    | { onDelete?: undefined; rowLabel?: (row: T) => string }
  );

export function DataTable<T extends { id: number }>({
  label,
  columns,
  data,
  onDelete,
  rowLabel,
  canDelete,
  emptyMessage,
  searchable = true,
  pageSize = 50,
}: DataTableProps<T>) {
  const state = useDataTableState(data, columns, pageSize);

  if (data.length === 0) {
    return (
      <div className="flex h-32 items-center justify-center">
        <p className="text-sm text-muted-foreground">{emptyMessage}</p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {searchable && (
        <div className="relative">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            type="text"
            placeholder="Rechercher..."
            aria-label={`Rechercher dans ${label}`}
            value={state.search}
            onChange={(e) => state.handleSearch(e.target.value)}
            className="flex h-9 w-full rounded-md border border-input bg-background pl-9 pr-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          />
        </div>
      )}

      <div className="overflow-x-auto">
        <table aria-label={label} className="w-full text-sm">
          <caption className="sr-only">{label}</caption>
          <thead>
            <tr className="border-b">
              {columns.map((col) => (
                <SortHeader
                  key={col.key}
                  label={col.label}
                  active={state.sortKey === col.key}
                  sortDir={state.sortDir}
                  onSort={() => state.handleSort(col.key)}
                />
              ))}
              {onDelete && (
                <th className="w-12 px-4 py-3">
                  <span className="sr-only">Actions</span>
                </th>
              )}
            </tr>
          </thead>
          <tbody>
            <TableRows
              rows={state.paged}
              columns={columns}
              onDelete={onDelete}
              rowLabel={rowLabel}
              canDelete={canDelete}
            />
          </tbody>
        </table>
      </div>

      {state.totalPages > 1 && (
        <PaginationBar
          total={state.sorted.length}
          filterNote={state.search.trim() ? ` (filtre sur ${data.length})` : null}
          safePage={state.safePage}
          totalPages={state.totalPages}
          onPrevious={state.previousPage}
          onNext={state.nextPage}
        />
      )}
    </div>
  );
}
