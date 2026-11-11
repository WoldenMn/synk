import type { ReactNode } from "react";

interface ChartA11yWrapperProps {
  /** Resume textuel de la serie, annonce par les lecteurs d'ecran. */
  label: string;
  children: ReactNode;
}

/**
 * Enveloppe un graphique Recharts dans une `figure` nommee par `aria-label`.
 * Le lecteur d'ecran annonce le resume ; le graphique reste atteignable au clavier
 * (`accessibilityLayer` de recharts) et les notes placees dans la figure sont lues.
 */
export function ChartA11yWrapper({ label, children }: ChartA11yWrapperProps) {
  return (
    <figure aria-label={label} className="m-0">
      {children}
    </figure>
  );
}
