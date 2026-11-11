import { Badge } from "@/components/ui/badge";
import { sourceLabel } from "../labels";

export function SourceBadge({ source }: { source: string }) {
  return (
    <Badge variant="outline" className="font-normal text-muted-foreground">
      {sourceLabel(source)}
    </Badge>
  );
}
