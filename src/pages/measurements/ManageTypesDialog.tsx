import { Plus, Trash } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { useConfirm } from "@/components/ui/confirm-dialog";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { useMeasurementTypes, useDeleteMeasurementType } from "@/features/manualMeasurements/hooks";

interface Props {
  manageOpen: boolean;
  setManageOpen: (open: boolean) => void;
  onAddType: () => void;
}

export function ManageTypesDialog({ manageOpen, setManageOpen, onAddType }: Props) {
  const { data: types } = useMeasurementTypes();
  const deleteTypeMutation = useDeleteMeasurementType();
  const confirm = useConfirm();

  return (
    <Dialog open={manageOpen} onOpenChange={setManageOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Gérer les types</DialogTitle>
          <DialogDescription>
            Ajoutez ou supprimez des types de mensuration personnalises
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-3 py-4 max-h-80 overflow-y-auto">
          {types?.map((t) => (
            <div key={t.id} className="flex items-center justify-between rounded-lg border p-3">
              <div>
                <p className="text-sm font-medium">{t.label}</p>
                <p className="text-xs text-muted-foreground">
                  {t.unit} {t.is_builtin === 1 ? "" : "- Personnalise"}
                </p>
              </div>
              {t.is_builtin === 0 && (
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 text-muted-foreground hover:text-destructive-text"
                  onClick={async () => {
                    if (
                      !(await confirm({
                        title: `Supprimer le type "${t.label}" et toutes ses mesures ?`,
                        destructive: true,
                      }))
                    )
                      return;
                    deleteTypeMutation.mutate(t.id);
                  }}
                  aria-label={`Supprimer le type ${t.label}`}
                >
                  <Trash className="h-4 w-4" />
                </Button>
              )}
            </div>
          ))}
        </div>
        <Separator />
        <Button
          variant="outline"
          className="w-full"
          onClick={() => {
            setManageOpen(false);
            onAddType();
          }}
        >
          <Plus className="mr-2 h-4 w-4" />
          Ajouter un type personnalise
        </Button>
      </DialogContent>
    </Dialog>
  );
}
