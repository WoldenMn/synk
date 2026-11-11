import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";

interface Props {
  total: number;
  filterNote: string | null;
  safePage: number;
  totalPages: number;
  onPrevious: () => void;
  onNext: () => void;
}

export function PaginationBar({
  total,
  filterNote,
  safePage,
  totalPages,
  onPrevious,
  onNext,
}: Props) {
  return (
    <div className="flex items-center justify-between pt-1">
      <p className="text-xs text-muted-foreground">
        {total} résultat{total > 1 ? "s" : ""}
        {filterNote}
      </p>
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="icon"
          className="h-8 w-8"
          disabled={safePage === 0}
          onClick={onPrevious}
          aria-label="Page précédente"
        >
          <ChevronLeft className="h-4 w-4" />
        </Button>
        <span className="text-sm text-muted-foreground">
          {safePage + 1} / {totalPages}
        </span>
        <Button
          variant="outline"
          size="icon"
          className="h-8 w-8"
          disabled={safePage >= totalPages - 1}
          onClick={onNext}
          aria-label="Page suivante"
        >
          <ChevronRight className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
