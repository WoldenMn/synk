import { Link } from "react-router-dom";
import type { KeyMeasurement } from "@/features/profileCard/types";
import { formatMeasure } from "@/features/profileCard/format";

export function MeasurementsSection({ measurements }: { measurements: KeyMeasurement[] }) {
  return (
    <section>
      <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
        Mensurations clés
      </h2>
      {measurements.length === 0 ? (
        <p className="text-sm text-muted-foreground">Aucune mensuration enregistrée.</p>
      ) : (
        <dl className="grid grid-cols-2 gap-x-8 gap-y-1.5 text-sm">
          {measurements.map((m) => (
            <div key={m.slug} className="flex justify-between border-b py-1">
              <dt className="text-muted-foreground">{m.label}</dt>
              <dd className="font-medium tabular-nums">{formatMeasure(m.value, "cm")}</dd>
            </div>
          ))}
        </dl>
      )}
      <Link
        to="/measurements"
        className="text-xs text-primary-text hover:underline mt-2 inline-block"
      >
        Voir les 47 mensurations
      </Link>
    </section>
  );
}
