import { Link } from "react-router-dom";
import type { Health } from "@/features/profileCard/types";

export function HealthSection({ health }: { health: Health }) {
  return (
    <section>
      <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
        Traitements en cours
      </h2>
      {health.active_treatments.length === 0 ? (
        <p className="text-sm text-muted-foreground">Aucun traitement en cours.</p>
      ) : (
        <ul className="text-sm space-y-1.5">
          {health.active_treatments.map((treatment) => (
            <li key={treatment} className="border-b py-1 font-medium">
              {treatment}
            </li>
          ))}
        </ul>
      )}
      <Link
        to="/medications"
        className="text-xs text-primary-text hover:underline mt-2 inline-block"
      >
        Gérer les traitements
      </Link>
    </section>
  );
}
