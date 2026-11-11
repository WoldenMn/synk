import type { Brand } from "./types";

// Grille HAUT HOMME Adidas (sportswear) — tour de poitrine (cm) -> taille alpha.
// maxValue = borne haute poitrine couverte par la taille (croissant). Sources :
// https://www.adidas.com/us/help/size_charts
// https://www.tactics.com/sizing/adidas-mens-tops-size-chart
// https://www.scheels.com/size-chart/adidas-mens-apparel-size-chart/
export const adidas: Brand = {
  id: "adidas",
  label: "Adidas",
  grids: {
    top: [
      { maxValue: 86, label: "XS" },
      { maxValue: 92, label: "S" },
      { maxValue: 100, label: "M" },
      { maxValue: 108, label: "L" },
      { maxValue: 118, label: "XL" },
      { maxValue: 999, label: "XXL" },
    ],
  },
};
