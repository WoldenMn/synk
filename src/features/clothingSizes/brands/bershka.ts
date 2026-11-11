import type { Brand } from "./types";

// Haut homme (tour de poitrine cm -> taille alpha Bershka). Points centraux sources
// concordants : XS 87, S 93, M 99, L 105, XL 111, XXL 117 cm. Bornes hautes = milieu
// entre deux points centraux. Sources :
//   https://www.matchnfit.com/marques/bershka.html
//   https://www.looksize.com/brand-size-chart/bershka
//   https://www.orangejeans.co/pages/bershka-size-guide (34.25/36.61/38.97/41.33/43.7/46.06 in x 2.54)
// Pantalon (tour de taille cm) NON ENCODE : les deux seules lectures chiffrees disponibles
// divergent (LookSize 70/74/78/82/86 vs 71/76/81/86/91 pour EU 34-42) et LookSize contient
// une coquille (EU44 = 86, identique a EU42). Faute de 3 sources chiffrees concordantes, la
// grille pants est laissee absente plutot que fabriquee.
export const bershka: Brand = {
  id: "bershka",
  label: "Bershka",
  grids: {
    top: [
      { maxValue: 90, label: "XS" },
      { maxValue: 96, label: "S" },
      { maxValue: 102, label: "M" },
      { maxValue: 108, label: "L" },
      { maxValue: 114, label: "XL" },
      { maxValue: 999, label: "XXL" },
    ],
  },
};
