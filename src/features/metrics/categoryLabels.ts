import type { MetricCategory } from "./types";

export const METRIC_CATEGORY_LABELS: Record<MetricCategory, string> = {
  body_composition: "Composition corporelle",
  circumference: "Mensurations",
  body_scan: "Constantes de santé",
  derived: "Dérivées",
  custom: "Personnalisées",
};

export const METRIC_CATEGORIES = Object.keys(METRIC_CATEGORY_LABELS) as MetricCategory[];
