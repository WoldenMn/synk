import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Trash } from "lucide-react";
import { toast } from "sonner";
import { notifyError, reportError } from "@/lib/reportError";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { usePurgeAuditLog } from "@/features/audit/hooks";

export function PurgeSection() {
  const purgeMutation = usePurgeAuditLog();
  const confirm = useConfirm();
  const [olderThanDays, setOlderThanDays] = useState(90);

  async function handlePurge() {
    if (olderThanDays < 0) {
      notifyError("La durée doit être positive");
      return;
    }
    if (
      !(await confirm({
        title: `Supprimer les entrées d'audit log plus anciennes que ${olderThanDays} jours ?`,
        description: "Cette action est définitive.",
        destructive: true,
      }))
    ) {
      return;
    }
    purgeMutation.mutate(olderThanDays, {
      onSuccess: (deleted) => {
        toast.success(`${deleted} entrée(s) supprimée(s)`);
      },
      onError: (e) => {
        reportError(e, {
          source: "mutation",
          context: "audit_purge",
          prefix: "Échec de la purge : ",
        });
      },
    });
  }

  return (
    <div className="border-t pt-3">
      <p className="text-sm font-medium mb-2 flex items-center gap-1">
        <Trash className="h-4 w-4" />
        Purge audit log
      </p>
      <div className="flex items-end gap-2">
        <div className="flex-1">
          <label htmlFor="purge-days" className="text-xs text-muted-foreground">
            Plus anciens que (jours)
          </label>
          <Input
            id="purge-days"
            type="number"
            min={0}
            value={olderThanDays}
            onChange={(e) => setOlderThanDays(Number(e.target.value) || 0)}
          />
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={handlePurge}
          disabled={purgeMutation.isPending}
        >
          {purgeMutation.isPending ? "Purge..." : "Purger"}
        </Button>
      </div>
    </div>
  );
}
