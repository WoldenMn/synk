import { Link } from "react-router-dom";
import type { Body } from "@/features/profileCard/types";
import { formatMeasure, formatProfileDate } from "@/features/profileCard/format";
import { formatNumber } from "@/lib/formatters";

export function BodySection({ body }: { body: Body }) {
  const rows: [string, string][] = [
    ["Âge", body.age != null ? `${body.age} ans` : "—"],
    ["Sexe", body.sex ?? "—"],
    ["Taille", formatMeasure(body.height_cm, "cm")],
    ["Poids", formatMeasure(body.weight_kg, "kg")],
    ["IMC", body.bmi != null ? formatNumber(body.bmi, 1) : "—"],
    ["Masse grasse", formatMeasure(body.fat_ratio, "%")],
    ["Masse musculaire", formatMeasure(body.muscle_mass, "kg")],
    ["Hydratation", formatMeasure(body.hydration, "kg")],
  ];

  return (
    <section>
      <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
        Corps
      </h2>
      <dl className="grid grid-cols-2 gap-x-8 gap-y-1.5 text-sm">
        {rows.map(([label, value]) => (
          <div key={label} className="flex justify-between border-b py-1">
            <dt className="text-muted-foreground">{label}</dt>
            <dd className="font-medium tabular-nums">{value}</dd>
          </div>
        ))}
      </dl>
      <p className="text-xs text-muted-foreground mt-2">
        {body.measured_at
          ? `Dernière pesée le ${formatProfileDate(body.measured_at)}`
          : "Aucune pesée enregistrée"}
      </p>
      {(body.age == null || body.sex == null || body.height_cm == null) && (
        <p className="mt-2 text-xs">
          <Link to="/settings" className="text-primary-text hover:underline">
            Renseigner dans Paramètres
          </Link>
        </p>
      )}
      <Link to="/body" className="text-xs text-primary-text hover:underline mt-2 inline-block">
        Voir la composition corporelle
      </Link>
    </section>
  );
}
