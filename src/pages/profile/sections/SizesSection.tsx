import { Link } from "react-router-dom";
import type { SizeEntry } from "@/features/profileCard/types";

const CATEGORY_LABELS: Record<string, string> = {
  top: "Haut",
  shirt: "Chemise",
  jacket: "Veste",
  pants: "Pantalon",
  shoes: "Pointure",
};

export function SizesSection({ sizes }: { sizes: SizeEntry[] }) {
  return (
    <section>
      <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
        Tailles
      </h2>
      {sizes.length === 0 ? (
        <p className="text-sm text-muted-foreground">Aucune taille enregistrée.</p>
      ) : (
        <dl className="grid grid-cols-2 gap-x-8 gap-y-1.5 text-sm">
          {sizes.map((s) => (
            <div key={s.category} className="flex justify-between border-b py-1">
              <dt className="text-muted-foreground">{CATEGORY_LABELS[s.category] ?? s.category}</dt>
              <dd className="font-medium">{s.value}</dd>
            </div>
          ))}
        </dl>
      )}
      <Link
        to="/measurements"
        className="text-xs text-primary-text hover:underline mt-2 inline-block"
      >
        Assistant Tailles
      </Link>
    </section>
  );
}
