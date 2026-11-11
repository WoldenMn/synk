import { Trash } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { Column } from "./types";

interface Props<T extends { id: number }> {
  rows: T[];
  columns: Column<T>[];
  onDelete?: (id: number) => void;
  rowLabel?: (row: T) => string;
  canDelete?: (row: T) => boolean;
}

export function TableRows<T extends { id: number }>({
  rows,
  columns,
  onDelete,
  rowLabel,
  canDelete,
}: Props<T>) {
  if (rows.length === 0) {
    return (
      <tr>
        <td
          colSpan={columns.length + (onDelete ? 1 : 0)}
          className="px-4 py-8 text-center text-muted-foreground"
        >
          Aucun résultat
        </td>
      </tr>
    );
  }
  return (
    <>
      {rows.map((row) => (
        <tr key={row.id} className="border-b last:border-0 hover:bg-muted/50">
          {columns.map((col) => (
            <td key={col.key} className="px-4 py-3">
              {col.render
                ? col.render(row)
                : String((row as Record<string, unknown>)[col.key] ?? "—")}
            </td>
          ))}
          {onDelete && (
            <td className="px-4 py-3">
              {(!canDelete || canDelete(row)) && (
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 text-muted-foreground hover:text-destructive-text"
                  onClick={() => onDelete(row.id)}
                  aria-label={rowLabel ? `Supprimer ${rowLabel(row)}` : "Supprimer la ligne"}
                >
                  <Trash className="h-4 w-4" />
                </Button>
              )}
            </td>
          )}
        </tr>
      ))}
    </>
  );
}
