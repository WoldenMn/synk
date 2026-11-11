import { Trash } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { formatDate } from "@/lib/formatters";

interface Props {
  entry: { id: number; date: string; content: string; tags?: string | null };
  index: number;
  onDelete: (id: number) => void;
}

export function JournalEntry({ entry, index: idx, onDelete }: Props) {
  return (
    <div
      key={entry.id}
      className="animate-fade-in-up"
      style={{ animationDelay: `${Math.min(idx * 30, 300)}ms` }}
    >
      <Card className="group">
        <CardContent className="pt-6">
          <div className="flex items-start justify-between gap-4">
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-sm font-semibold">{formatDate(entry.date)}</span>
                {entry.tags && (
                  <div className="flex gap-1 flex-wrap">
                    {entry.tags.split(",").map((t, i) => (
                      <span
                        key={i}
                        className="text-xs uppercase tracking-wider bg-primary/10 text-primary-text px-2 py-0.5 rounded-full"
                      >
                        {t.trim()}
                      </span>
                    ))}
                  </div>
                )}
              </div>
              <p className="text-sm whitespace-pre-wrap">{entry.content}</p>
            </div>
            <Button
              variant="ghost"
              size="icon"
              className="opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus-visible:opacity-100 transition-opacity"
              onClick={() => onDelete(entry.id)}
              aria-label="Supprimer l'entrée du journal"
            >
              <Trash className="h-4 w-4 text-destructive-text" />
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
