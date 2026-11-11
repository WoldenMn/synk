import type { Brand } from "./types";

// Grille HAUT HOMME Nike sportswear (t-shirt/sweat) : tour de poitrine (cm) -> taille alpha.
// Bornes hautes = guide officiel Nike (chest cm), corroborees par 2 sources independantes.
// Sources :
//   https://www.nike.com/gb/size-fit/mens-tops-alpha
//   https://www.size.ly/size-chart/nike
//   https://sizecharter.com/brands/nike/mens
export const nike: Brand = {
  id: "nike",
  label: "Nike",
  grids: {
    top: [
      { maxValue: 88, label: "XS" },
      { maxValue: 96, label: "S" },
      { maxValue: 104, label: "M" },
      { maxValue: 112, label: "L" },
      { maxValue: 124, label: "XL" },
      { maxValue: 999, label: "XXL" },
    ],
  },
};
