import { useNavigate } from "react-router-dom";
import { TriangleAlert } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { IntegrityWarningPayload } from "@/hooks/appReady/types";

export function IntegrityBanner({ warning }: { warning?: IntegrityWarningPayload }) {
  const navigate = useNavigate();
  if (!warning) return null;
  return (
    <div
      role="alert"
      className="mb-4 flex flex-wrap items-center gap-3 rounded-lg border border-destructive/40 bg-destructive/5 px-4 py-3 text-sm"
    >
      <TriangleAlert className="h-4 w-4 shrink-0" aria-hidden="true" />
      <span className="font-medium">La base de données a échoué au contrôle d'intégrité.</span>
      <span className="text-muted-foreground">{warning.Details}</span>
      <Button size="sm" variant="outline" onClick={() => navigate("/settings")}>
        Voir les sauvegardes
      </Button>
    </div>
  );
}
