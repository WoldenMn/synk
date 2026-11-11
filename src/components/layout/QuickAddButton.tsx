import { lazy, Suspense, useId, useState, useRef, useEffect } from "react";
import { Plus, Scale, Heart, Ruler } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { SHORTCUTS } from "@/lib/shortcuts";
import { useUiStore } from "@/stores/uiStore";
import { cn } from "@/lib/utils";

const ManualWeightForm = lazy(() =>
  import("@/features/bodyMeasures/components/ManualWeightForm").then((m) => ({
    default: m.ManualWeightForm,
  }))
);
const ManualBPForm = lazy(() =>
  import("@/features/bloodPressure/components/ManualBPForm").then((m) => ({
    default: m.ManualBPForm,
  }))
);
const ManualMeasurementForm = lazy(() =>
  import("@/features/manualMeasurements/components/ManualMeasurementForm").then((m) => ({
    default: m.ManualMeasurementForm,
  }))
);

const QUICK_ADD_OPTIONS = [
  { label: "Poids", icon: Scale, action: "weight" },
  { label: "Tension", icon: Heart, action: "blood_pressure" },
  { label: "Mensuration", icon: Ruler, action: "measurement" },
] as const;

type DialogType = "weight" | "blood_pressure" | "measurement" | null;

export function QuickAddButton() {
  const open = useUiStore((s) => s.quickAddOpen);
  const setOpen = useUiStore((s) => s.setQuickAddOpen);
  const menuRef = useRef<HTMLDivElement>(null);
  const menuId = useId();
  const triggerRef = useRef<HTMLButtonElement>(null);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const [activeDialog, setActiveDialog] = useState<DialogType>(null);
  const [mounted, setMounted] = useState<ReadonlySet<string>>(() => new Set());

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    if (open) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open, setOpen]);

  useEffect(() => {
    if (open) itemRefs.current[0]?.focus();
  }, [open]);

  function closeMenu() {
    setOpen(false);
    triggerRef.current?.focus();
  }

  function handleMenuKeyDown(e: React.KeyboardEvent) {
    const items = itemRefs.current.filter((el): el is HTMLButtonElement => el !== null);
    const index = items.findIndex((el) => el === document.activeElement);
    if (e.key === "Escape") {
      e.preventDefault();
      closeMenu();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      items[(index + 1) % items.length]?.focus();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      items[(index - 1 + items.length) % items.length]?.focus();
    }
  }

  function handleOptionClick(action: string) {
    setActiveDialog(action as DialogType);
    setMounted((current) => new Set(current).add(action));
    setOpen(false);
  }

  return (
    <>
      <div ref={menuRef} className="fixed bottom-6 right-6 z-40">
        <Tooltip content={`Ajout rapide (${SHORTCUTS.quickAdd.label})`} side="left">
          <Button
            size="icon"
            className={cn(
              "h-12 w-12 rounded-full shadow-lg transition-transform",
              "bg-primary hover:bg-primary/90 text-primary-foreground",
              open && "rotate-45"
            )}
            ref={triggerRef}
            onClick={() => setOpen(!open)}
            aria-label={open ? "Fermer le menu d'ajout rapide" : "Ajout rapide"}
            aria-haspopup="menu"
            aria-controls={open ? menuId : undefined}
            aria-expanded={open}
            aria-keyshortcuts={SHORTCUTS.quickAdd.keyshortcuts}
          >
            <Plus className="h-6 w-6" />
          </Button>
        </Tooltip>

        {open && (
          <div
            id={menuId}
            role="menu"
            aria-label="Ajout rapide"
            tabIndex={-1}
            onKeyDown={handleMenuKeyDown}
            className="absolute bottom-14 right-0 flex flex-col gap-2 animate-in slide-in-from-bottom-2 fade-in-0"
          >
            {QUICK_ADD_OPTIONS.map((opt, i) => (
              <button
                key={opt.action}
                type="button"
                role="menuitem"
                ref={(el) => {
                  itemRefs.current[i] = el;
                }}
                onClick={() => handleOptionClick(opt.action)}
                className={cn(
                  "flex items-center gap-2 rounded-lg border bg-card px-4 py-2.5 text-sm font-medium shadow-lg",
                  "hover:bg-accent/15 transition-colors whitespace-nowrap"
                )}
              >
                <opt.icon className="h-4 w-4" />
                {opt.label}
              </button>
            ))}
          </div>
        )}
      </div>

      <Suspense fallback={null}>
        {mounted.has("weight") && (
          <ManualWeightForm
            open={activeDialog === "weight"}
            onOpenChange={(v) => !v && setActiveDialog(null)}
          />
        )}
        {mounted.has("blood_pressure") && (
          <ManualBPForm
            open={activeDialog === "blood_pressure"}
            onOpenChange={(v) => !v && setActiveDialog(null)}
          />
        )}
        {mounted.has("measurement") && (
          <ManualMeasurementForm
            open={activeDialog === "measurement"}
            onOpenChange={(v) => !v && setActiveDialog(null)}
          />
        )}
      </Suspense>
    </>
  );
}
