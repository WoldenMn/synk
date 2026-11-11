import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { useSetClothingSize, useDeleteClothingSize } from "../hooks";

interface SizeEditDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  category: string;
  label: string;
  current: string | null;
  source: "override" | "manual";
}

export function SizeEditDialog({
  open,
  onOpenChange,
  category,
  label,
  current,
  source,
}: SizeEditDialogProps) {
  const [value, setValue] = useState("");
  const setMutation = useSetClothingSize();
  const deleteMutation = useDeleteClothingSize();

  useEffect(() => {
    if (open) setValue(current ?? "");
  }, [open, current]);

  function handleSave() {
    if (!value.trim()) return;
    setMutation.mutate(
      { category, value: value.trim(), source },
      { onSuccess: () => onOpenChange(false) }
    );
  }

  function handleReset() {
    deleteMutation.mutate(category, { onSuccess: () => onOpenChange(false) });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{label}</DialogTitle>
        </DialogHeader>
        <div className="space-y-2 py-4">
          <Label htmlFor="size-value">
            {source === "manual" ? "Ta taille" : "Ta taille réelle"}
          </Label>
          <Input
            id="size-value"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="Ex : 52"
          />
        </div>
        <DialogFooter>
          {current != null && (
            <Button variant="ghost" onClick={handleReset}>
              Réinitialiser
            </Button>
          )}
          <Button onClick={handleSave} disabled={setMutation.isPending || !value.trim()}>
            Enregistrer
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
