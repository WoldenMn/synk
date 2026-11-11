import { Pencil } from "lucide-react";

export type SizeBadge = "estimé" | "votre taille" | "saisi";

interface SizeCardProps {
  label: string;
  mainValue: string | null;
  badge: SizeBadge | null;
  conversions?: string;
  note?: string;
  alert?: string;
  emptyLabel: string;
  editable: boolean;
  onEdit: () => void;
}

const BADGE_CLS: Record<SizeBadge, string> = {
  estimé: "bg-warning/15 text-warning-text",
  "votre taille": "bg-success/15 text-success-text",
  saisi: "bg-muted text-muted-foreground",
};

export function SizeCard({
  label,
  mainValue,
  badge,
  conversions,
  note,
  alert,
  emptyLabel,
  editable,
  onEdit,
}: SizeCardProps) {
  return (
    <div className="relative rounded-lg border bg-card p-4">
      {editable && (
        <button
          onClick={onEdit}
          className="absolute right-3 top-3 text-muted-foreground hover:text-foreground"
          aria-label={`Modifier la taille ${label}`}
        >
          <Pencil className="h-4 w-4" />
        </button>
      )}
      <p className="text-xs uppercase tracking-wider text-muted-foreground">{label}</p>

      {mainValue == null ? (
        editable ? (
          <button onClick={onEdit} className="mt-2 text-sm text-primary-text hover:underline">
            {emptyLabel}
          </button>
        ) : (
          <p className="mt-2 text-sm text-muted-foreground">{emptyLabel}</p>
        )
      ) : (
        <>
          <div className="mt-1 flex items-center gap-2">
            <span className="text-2xl font-bold">{mainValue}</span>
            {badge && (
              <span className={`rounded px-1.5 py-0.5 text-xs font-semibold ${BADGE_CLS[badge]}`}>
                {badge}
              </span>
            )}
          </div>
          {conversions && <p className="mt-1 text-xs text-muted-foreground">{conversions}</p>}
          {alert && <p className="mt-1 text-xs text-warning-text">↕ {alert}</p>}
          {note && <p className="mt-1.5 text-xs text-muted-foreground">{note}</p>}
        </>
      )}
    </div>
  );
}
