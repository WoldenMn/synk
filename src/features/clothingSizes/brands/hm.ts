import type { Brand } from "./types";

// Haut homme (tour de poitrine cm -> taille alpha H&M).
// Bornes hautes concordantes des grilles officielles H&M (poitrine) : 84 / 92 / 100 / 108 / 116.
// Sources : https://www.sizedepo.com/sc/sizechart/hm-men-208 ; https://sizecharter.com/brands/h-m/mens ; https://whatismysize.com/clothing/hm-clothing-size-chart
export const hm: Brand = {
  id: "hm",
  label: "H&M",
  grids: {
    top: [
      { maxValue: 84, label: "XS" },
      { maxValue: 92, label: "S" },
      { maxValue: 100, label: "M" },
      { maxValue: 108, label: "L" },
      { maxValue: 116, label: "XL" },
      { maxValue: 999, label: "XXL" },
    ],
    // Pantalon homme (tour de taille cm -> taille H&M, correspondance EU numerique).
    // Grille officielle H&M pantalon : 28->72, 30->76, 32->80, 33->84, 36->92, 38->96, 40->100, 42->104, 44->108.
    // Sources : https://www.sizedepo.com/sc/chart/hm-men-pants-and-trousers/861 ; https://sizecharter.com/brands/h-m/mens ; https://whatismysize.com/clothing/hm-clothing-size-chart
    pants: [
      { maxValue: 72, label: "28" },
      { maxValue: 76, label: "30" },
      { maxValue: 80, label: "32" },
      { maxValue: 84, label: "33" },
      { maxValue: 92, label: "36" },
      { maxValue: 96, label: "38" },
      { maxValue: 100, label: "40" },
      { maxValue: 104, label: "42" },
      { maxValue: 999, label: "44" },
    ],
  },
};
