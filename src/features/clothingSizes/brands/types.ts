import type { CategoryKey } from "../types";
import type { ResolveOpts } from "../sizing/opts";

// Une bande de taille propre à une marque : borne haute de mesure -> label affiché.
export interface BrandBand {
  maxValue: number;
  label: string;
}

export interface Brand {
  id: string;
  label: string;
  // Une marque n'expose que les catégories où elle a une grille propre.
  grids: Partial<Record<CategoryKey, BrandBand[]>>;
}

export interface BrandSizeResult {
  label: string;
  nearThreshold?: { size: string; direction: "down" | "up" };
}

// Bornes de mesure + marge « entre deux tailles » par catégorie, alignées sur l'existant
// (sizing/index.ts). Mesure source : top<-chest, shirt<-neck, pants<-waist.
export const BRAND_RESOLVE_OPTS: Record<CategoryKey, ResolveOpts> = {
  top: { min: 70, max: 140, margin: 2 },
  shirt: { min: 33, max: 50, margin: 0.5 },
  jacket: { min: 70, max: 140, margin: 1 },
  pants: { min: 55, max: 140, margin: 1.5 },
  shoes: { min: 0, max: 0, margin: 0 }, // non utilise en v1 (sneakers hors scope)
};
