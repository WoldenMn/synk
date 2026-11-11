const EMPTY_LABEL = "Graphique vide, aucune donnée";

function formatBound(value: number, unit: string): string {
  const rounded = Number.isInteger(value) ? value : Number(value.toFixed(1));
  const text = rounded.toLocaleString("fr-FR");
  return unit ? `${text} ${unit}` : text;
}

function plural(count: number, singular: string, suffix = "s"): string {
  return count > 1 ? `${singular}${suffix}` : singular;
}

interface RangeLabelOptions {
  /** Intitule de la courbe, ex. "Courbe du poids". */
  title: string;
  /** Valeurs numeriques de la serie (deja filtrees des null). */
  values: number[];
  /** Unite affichee apres min/max, ex. "kg". Vide si sans unite. */
  unit?: string;
  /** Nom de l'unite de comptage des points, ex. "mesure", "point", "jour". */
  pointNoun?: string;
}

/**
 * Construit un aria-label resume pour un graphique a serie unique :
 * "Courbe du poids, 12 points, de 70 a 75 kg".
 * Renvoie un label "vide" si la serie ne contient aucune valeur.
 */
export function rangeChartLabel({
  title,
  values,
  unit = "",
  pointNoun = "point",
}: RangeLabelOptions): string {
  if (values.length === 0) return EMPTY_LABEL;

  const count = values.length;
  const countText = `${count} ${plural(count, pointNoun)}`;

  const min = Math.min(...values);
  const max = Math.max(...values);
  const rangeText =
    min === max
      ? `valeur ${formatBound(min, unit)}`
      : `de ${formatBound(min, "")} a ${formatBound(max, unit)}`;

  return `${title}, ${countText}, ${rangeText}`;
}

interface CountLabelOptions {
  /** Intitule du graphique, ex. "Tension artérielle systolique et diastolique". */
  title: string;
  /** Nombre d'elements representes. */
  count: number;
  /** Nom de l'unite de comptage, ex. "mesure", "entrainement", "nuit". */
  pointNoun: string;
}

/**
 * Construit un aria-label resume pour un graphique multi-series ou categoriel
 * ou seul le nombre d'elements est pertinent :
 * "Tension artérielle systolique et diastolique, 8 mesures".
 */
export function countChartLabel({ title, count, pointNoun }: CountLabelOptions): string {
  if (count === 0) return EMPTY_LABEL;
  return `${title}, ${count} ${plural(count, pointNoun)}`;
}
