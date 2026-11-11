import type { Brand } from "./types";

// Grille jean homme Levi's — tour de taille corps (cm) -> taille W.
// Bornes = borne haute du tour de taille corps par size, guide officiel Levi's
// (plages par pas de 2 : W28 71-74, W30 76-79, W32 81-84, W34 86-89, W36 91-94, W38 97-99).
// Tailles impaires interpolees et confirmees par le tableau corps footshop.
// Sources :
//   https://www.levi.com/US/en_US/info/sizeguide
//   https://www.size.ly/size-chart/levis
//   https://www.footshop.eu/en/size-charts/levi-s
export const levis: Brand = {
  id: "levis",
  label: "Levi's",
  grids: {
    pants: [
      { maxValue: 74, label: "W28" },
      { maxValue: 76, label: "W29" },
      { maxValue: 79, label: "W30" },
      { maxValue: 81, label: "W31" },
      { maxValue: 84, label: "W32" },
      { maxValue: 86, label: "W33" },
      { maxValue: 89, label: "W34" },
      { maxValue: 94, label: "W36" },
      { maxValue: 99, label: "W38" },
      { maxValue: 999, label: "W40" },
    ],
  },
};
